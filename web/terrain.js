const NASA_SOURCE_ID = "nasa-blue-marble";
const TERRAIN_SOURCE_ID = "aws-terrarium-dem";
const ROUTE_SOURCE_ID = "journey-route";
const ROUTE_LAYER_ID = "journey-route-line";
const MAP_MAX_ZOOM = 12;
const IMAGE_MAX_ZOOM = 8;
const FOLLOW_MAX_ZOOM = 7.8;
const DEFAULT_CENTER = [32.5, 36.2];
const DEFAULT_ZOOM = 4.2;
const EMPTY_ROUTE = { type: "FeatureCollection", features: [] };

let mapLibreModulePromise;

function loadMapLibre() {
  if (!mapLibreModulePromise) {
    mapLibreModulePromise = import("./vendor/maplibre/maplibre-gl.mjs").catch((error) => {
      mapLibreModulePromise = null;
      throw error;
    });
  }
  return mapLibreModulePromise;
}

function validCoordinates(point) {
  const lat = Number(point?.lat);
  const lon = Number(point?.lon);
  if (!Number.isFinite(lat) || !Number.isFinite(lon) || lat < -85.051129 || lat > 85.051129 || lon < -180 || lon > 180) {
    return null;
  }
  return { lat, lon };
}

function placeId(place, coordinates) {
  if (place?.id !== null && place?.id !== undefined && String(place.id).length) return String(place.id);
  return `${String(place?.name || "Place")}@${coordinates.lat},${coordinates.lon}`;
}

function normalizeState(state) {
  const places = new Map();
  const stops = [];
  const rawPlaces = Array.isArray(state?.places) ? state.places : [];
  const rawStops = Array.isArray(state?.stops) ? state.stops : [];
  const routeValid = rawStops.length >= 2 && rawStops.every((stop) => validCoordinates(stop));
  const settingCoordinates = validCoordinates(state?.setting);
  const settingId = state?.setting?.id === null || state?.setting?.id === undefined
    ? null
    : String(state.setting.id);

  for (const place of rawPlaces) {
    const coordinates = validCoordinates(place);
    if (!coordinates || !place || typeof place !== "object") continue;
    const id = placeId(place, coordinates);
    places.set(id, {
      id,
      name: String(place.name || "Unnamed place"),
      lat: coordinates.lat,
      lon: coordinates.lon,
      payload: { ...place, id, lat: coordinates.lat, lon: coordinates.lon },
      isRouteStop: false,
      isActive: id === settingId,
    });
  }

  for (const stop of rawStops) {
    const coordinates = validCoordinates(stop);
    if (!coordinates || !stop || typeof stop !== "object") continue;
    const id = placeId(stop, coordinates);
    const normalizedStop = {
      id,
      name: String(stop.name || "Unnamed route stop"),
      lat: coordinates.lat,
      lon: coordinates.lon,
      payload: { ...stop, id, lat: coordinates.lat, lon: coordinates.lon },
      isRouteStop: true,
      isActive: id === settingId,
    };
    stops.push(normalizedStop);
    const existing = places.get(id);
    if (existing) {
      existing.isRouteStop = true;
      existing.isActive ||= normalizedStop.isActive;
      existing.payload = { ...existing.payload, ...normalizedStop.payload, isRouteStop: true };
    } else {
      places.set(id, normalizedStop);
    }
  }

  if (settingCoordinates) {
    const id = settingId || placeId(state.setting, settingCoordinates);
    const existing = places.get(id);
    if (existing) {
      existing.isActive = true;
    } else {
      places.set(id, {
        id,
        name: String(state.setting.name || "Current setting"),
        lat: settingCoordinates.lat,
        lon: settingCoordinates.lon,
        payload: { ...state.setting, id, lat: settingCoordinates.lat, lon: settingCoordinates.lon },
        isRouteStop: false,
        isActive: true,
      });
    }
  }

  return {
    places: Array.from(places.values()),
    stops,
    routeValid,
    setting: settingCoordinates
      ? { ...state.setting, id: settingId, lat: settingCoordinates.lat, lon: settingCoordinates.lon }
      : null,
    settingId,
    journeyTitle: typeof state?.journey_title === "string" ? state.journey_title : null,
    book: Number.isFinite(Number(state?.book)) ? Number(state.book) : null,
    chapter: Number.isFinite(Number(state?.chapter)) ? Number(state.chapter) : null,
    verse: Number.isFinite(Number(state?.verse)) ? Number(state.verse) : null,
  };
}

function routeFeature(state) {
  if (state.stops.length < 2 || !state.routeValid) return EMPTY_ROUTE;
  const rawStops = state.stops;
  // Keep the route in the supplied order, and do not bridge across invalid stops.
  const validStops = rawStops.map((stop) => validCoordinates(stop));
  if (validStops.length < 2 || validStops.some((stop) => !stop)) return EMPTY_ROUTE;
  return {
    type: "FeatureCollection",
    features: [{
      type: "Feature",
      properties: {},
      geometry: {
        type: "LineString",
        coordinates: validStops.map(({ lon, lat }) => [lon, lat]),
      },
    }],
  };
}

function distanceSquared(place, center) {
  const latScale = Math.cos(center.lat * Math.PI / 180);
  const dx = (place.lon - center.lon) * latScale;
  const dy = place.lat - center.lat;
  return dx * dx + dy * dy;
}

function visiblePlaces(state, center) {
  if (state.places.length <= 100) return state.places;
  const selected = new Map();
  const add = (place) => {
    if (place && selected.size < 100) selected.set(place.id, place);
  };
  const byId = new Map(state.places.map((place) => [place.id, place]));
  add(state.places.find((place) => place.isActive));
  add(state.stops[0] && byId.get(state.stops[0].id));
  add(state.stops[state.stops.length - 1] && byId.get(state.stops[state.stops.length - 1].id));
  for (const place of [...state.places].sort((a, b) => distanceSquared(a, center) - distanceSquared(b, center))) add(place);
  return Array.from(selected.values());
}

function currentBoundsPoints(state) {
  if (state.stops.length >= 2 && state.routeValid) return state.stops.map(({ lat, lon }) => ({ lat, lon }));
  if (state.setting) return [{ lat: state.setting.lat, lon: state.setting.lon }];
  const points = state.places.map(({ lat, lon }) => ({ lat, lon }));
  return points.slice(0, 100);
}

function attributionHTML() {
  return [
    '<a href="https://earthdata.nasa.gov/gibs/" target="_blank" rel="noopener">NASA EOSDIS GIBS imagery</a>',
    '<a href="https://registry.opendata.aws/terrain-tiles/" target="_blank" rel="noopener">Mapzen terrain tiles</a> · <a href="https://github.com/tilezen/joerd/blob/master/docs/attribution.md" target="_blank" rel="noopener">terrain attribution</a>',
  ].join(" · ");
}

export async function createTerrainMap(container, {
  onInteract = () => {},
  onPlaceSelect = () => {},
  onStatus = () => {},
} = {}) {
  let map = null;
  let maplibreApi = null;
  let active = true;
  let styleReady = false;
  let havePreviousSetting = false;
  let previousSettingId = null;
  let desired3D = false;
  let pendingFit = false;
  let pendingNorth = false;
  let latestState = normalizeState(null);
  let latestOptions = { follow: true, overview: false, animate: true, force: false };
  let baseFailed = false;
  let terrainFailed = false;
  let otherResourcesFailed = false;
  const markers = new Map();
  let detachObserver = null;

  const connected = () => !!container && container.isConnected;
  const reportStatus = (state, message) => {
    if (!active || !connected()) return;
    try {
      onStatus({ state, message });
    } catch {
      // Status presentation belongs to the host UI and must not break map cleanup.
    }
  };
  const notifyInteract = (event) => {
    if (!active || !connected() || !event?.originalEvent) return;
    try {
      onInteract();
    } catch {
      // Camera interaction remains usable if host bookkeeping fails.
    }
  };
  const readCamera = () => {
    if (!map || !active) return null;
    const center = map.getCenter();
    return {
      center: { lat: center.lat, lon: center.lng },
      zoom: map.getZoom(),
      bearing: map.getBearing(),
      pitch: map.getPitch(),
      terrainEnabled: desired3D,
      loaded: styleReady,
      baseImageryFailed: baseFailed,
      terrainFailed,
    };
  };
  const destroy = () => {
    if (!active) return;
    active = false;
    detachObserver?.disconnect();
    detachObserver = null;
    for (const record of markers.values()) {
      record.button.removeEventListener("click", record.click);
      record.marker.remove();
    }
    markers.clear();
    if (map) {
      try {
        map.remove();
      } catch {
        // MapLibre can already be torn down after a failed WebGL initialization.
      }
      map = null;
    }
  };
  const controller = {
    update(state, { follow = true, overview = false, animate = true, force = false } = {}) {
      if (!active) return;
      const resumedFollow = follow === true && latestOptions.follow === false;
      latestState = normalizeState(state);
      latestOptions = {
        follow: follow === true,
        overview: overview === true,
        animate: animate !== false,
        force: force === true || resumedFollow,
      };
      if (styleReady) applyLatestState();
    },
    fit() {
      if (!active) return;
      if (!styleReady) {
        pendingFit = true;
        return;
      }
      fitState(true);
    },
    set3D(enabled) {
      if (!active) return;
      desired3D = enabled === true;
      if (styleReady) apply3D(true);
    },
    zoom(delta) {
      if (!active || !Number.isFinite(Number(delta))) return;
      if (!styleReady) return;
      const targetZoom = Math.max(1.5, Math.min(MAP_MAX_ZOOM, map.getZoom() + Number(delta)));
      map.zoomTo(targetZoom, { duration: motionDuration(260) });
    },
    resetNorth() {
      if (!active) return;
      if (!styleReady) {
        pendingNorth = true;
        return;
      }
      map.rotateTo(0, { duration: motionDuration(360) });
    },
    destroy,
    getState: readCamera,
  };

  if (!container || typeof container.appendChild !== "function") {
    reportStatus("error", "The interactive map area is unavailable. Reopen the map to try again.");
    destroy();
    return controller;
  }
  if (!connected()) {
    destroy();
    return controller;
  }
  if (typeof MutationObserver === "function" && container.ownerDocument?.documentElement) {
    detachObserver = new MutationObserver(() => {
      if (!container.isConnected) destroy();
    });
    detachObserver.observe(container.ownerDocument.documentElement, { childList: true, subtree: true });
  }
  reportStatus("loading", "Loading the NASA Blue Marble map and terrain data…");

  function motionDuration(duration) {
    const view = container.ownerDocument?.defaultView;
    const reduced = view?.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true;
    return reduced ? 0 : duration;
  }

  function statusForResources() {
    if (!styleReady || !active) return;
    if (baseFailed && terrainFailed) {
      reportStatus("degraded", "NASA Blue Marble imagery and elevation data failed to load. Reopen the map to retry.");
    } else if (baseFailed) {
      reportStatus("degraded", "NASA Blue Marble imagery failed to load. Reopen the map to retry.");
    } else if (terrainFailed) {
      reportStatus("degraded", "Elevation data failed to load; the map remains available without 3D relief. Reopen the map to retry.");
    } else if (otherResourcesFailed) {
      reportStatus("degraded", "Some map resources failed to load. Reopen the map to retry.");
    } else if (map?.loaded()) {
      reportStatus("ready", desired3D ? "Modern imagery and 3D terrain are ready." : "NASA Blue Marble imagery is ready. Elevation relief loads when 3D is enabled.");
    }
  }

  function ensureMarker(place) {
    let record = markers.get(place.id);
    if (!record) {
      const button = container.ownerDocument.createElement("button");
      button.type = "button";
      button.className = "atlas-marker";
      button.setAttribute("aria-label", place.name);
      button.title = place.name;
      const dot = container.ownerDocument.createElement("span");
      dot.className = "atlas-marker-dot";
      dot.setAttribute("aria-hidden", "true");
      const label = container.ownerDocument.createElement("span");
      label.className = "atlas-marker-name";
      label.textContent = place.name;
      button.append(dot, label);
      let currentRecord = null;
      const click = (event) => {
        event.stopPropagation();
        if (!active || !connected()) return;
        try {
          onPlaceSelect(currentRecord.place.payload);
        } catch {
          // A selection callback is optional host behavior; leave the map intact.
        }
      };
      button.addEventListener("click", click);
      const marker = new maplibreApi.Marker({ element: button, anchor: "left", offset: [-10, 0] })
        .setLngLat([place.lon, place.lat])
        .addTo(map);
      record = { marker, button, label, click, place };
      currentRecord = record;
      markers.set(place.id, record);
    } else {
      record.place = place;
      record.marker.setLngLat([place.lon, place.lat]);
      record.label.textContent = place.name;
      record.button.setAttribute("aria-label", place.name);
      record.button.title = place.name;
    }
    record.marker.setOffset([place.isActive ? -14.5 : -10, 0]);
    record.button.classList.toggle("is-active", place.isActive);
    record.button.classList.toggle("is-route-stop", place.isRouteStop);
  }

  function updateMarkers() {
    if (!map || !styleReady || !active || !connected()) return;
    const centerPoint = map.getCenter();
    const center = { lat: centerPoint.lat, lon: centerPoint.lng };
    const nextPlaces = visiblePlaces(latestState, center);
    const nextIds = new Set(nextPlaces.map((place) => place.id));
    for (const [id, record] of markers) {
      if (!nextIds.has(id)) {
        record.button.removeEventListener("click", record.click);
        record.marker.remove();
        markers.delete(id);
      }
    }
    for (const place of nextPlaces) ensureMarker(place);
  }

  function fitState(animate) {
    if (!map || !styleReady || !active || !connected()) return;
    const points = currentBoundsPoints(latestState);
    if (!points.length) {
      map.flyTo({ center: DEFAULT_CENTER, zoom: DEFAULT_ZOOM, duration: motionDuration(650), essential: false });
      return;
    }
    const bounds = points.reduce((value, point) => ({
      minLat: Math.min(value.minLat, point.lat),
      maxLat: Math.max(value.maxLat, point.lat),
      minLon: Math.min(value.minLon, point.lon),
      maxLon: Math.max(value.maxLon, point.lon),
    }), { minLat: Infinity, maxLat: -Infinity, minLon: Infinity, maxLon: -Infinity });
    const { minLat, maxLat, minLon, maxLon } = bounds;
    const duration = animate ? motionDuration(720) : 0;
    if (minLat === maxLat && minLon === maxLon) {
      map.flyTo({ center: [minLon, minLat], zoom: Math.min(6.7, FOLLOW_MAX_ZOOM), duration, essential: false });
      return;
    }
    const dimensions = map.getContainer().getBoundingClientRect();
    const vertical = Math.max(22, Math.min(56, Math.round(Math.min(dimensions.width, dimensions.height) * 0.09)));
    const padding = dimensions.width > 850
      ? { top: vertical, right: Math.min(478, Math.round(dimensions.width * 0.34) + 48), bottom: vertical, left: 48 }
      : { top: 40, right: 40, bottom: 40, left: 40 };
    map.fitBounds([[minLon, minLat], [maxLon, maxLat]], {
      padding,
      maxZoom: FOLLOW_MAX_ZOOM,
      duration,
      essential: false,
    });
  }

  function apply3D(animate) {
    if (!map || !styleReady || !active || !connected()) return;
    try {
      if (desired3D) {
        map.setTerrain({ source: TERRAIN_SOURCE_ID, exaggeration: 1.15 });
        map.easeTo({ pitch: 56, duration: motionDuration(animate ? 600 : 0), essential: false });
      } else {
        map.setTerrain(null);
        map.easeTo({ pitch: 0, duration: motionDuration(animate ? 450 : 0), essential: false });
      }
    } catch {
      terrainFailed = true;
      statusForResources();
    }
  }

  function applyLatestState() {
    if (!map || !styleReady || !active || !connected()) return;
    try {
      map.getSource(ROUTE_SOURCE_ID)?.setData(routeFeature(latestState));
    } catch {
      otherResourcesFailed = true;
    }
    updateMarkers();

    const settingChanged = !havePreviousSetting || previousSettingId !== latestState.settingId;
    if (latestOptions.overview || (latestOptions.force && latestOptions.follow && !latestState.setting)) {
      fitState(latestOptions.animate);
    } else if (latestOptions.follow && latestState.setting && (settingChanged || latestOptions.force)) {
      const currentZoom = map.getZoom();
      const zoom = Math.max(7.2, Math.min(FOLLOW_MAX_ZOOM, currentZoom));
      const duration = latestOptions.animate ? motionDuration(850) : 0;
      map.flyTo({
        center: [latestState.setting.lon, latestState.setting.lat],
        zoom,
        offset: map.getContainer().getBoundingClientRect().width > 850
          ? [-Math.min(239, Math.round(map.getContainer().getBoundingClientRect().width * 0.17) + 24), 0]
          : [0, 0],
        duration,
        essential: false,
      });
    }
    previousSettingId = latestState.settingId;
    havePreviousSetting = true;
    const canvas = map.getCanvas();
    const cameraLabel = latestState.journeyTitle
      ? `Interactive map for ${latestState.journeyTitle}`
      : latestState.book !== null && latestState.chapter !== null
        ? `Interactive map for ${latestState.book}, chapter ${latestState.chapter}`
        : "Interactive map of biblical places";
    canvas.setAttribute("aria-label", cameraLabel);
    if (pendingFit) {
      pendingFit = false;
      fitState(true);
    }
    if (pendingNorth) {
      pendingNorth = false;
      map.rotateTo(0, { duration: motionDuration(360) });
    }
  }

  try {
    maplibreApi = await loadMapLibre();
    if (!active || !connected()) {
      destroy();
      return controller;
    }
    if (typeof maplibreApi.supported === "function" && !maplibreApi.supported()) {
      reportStatus("degraded", "This browser does not support WebGL maps. The reader and place list remain available.");
      destroy();
      return controller;
    }

    map = new maplibreApi.Map({
      container,
      style: {
        version: 8,
        sources: {
          [NASA_SOURCE_ID]: {
            type: "raster",
            tiles: ["https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/BlueMarble_ShadedRelief_Bathymetry/default/GoogleMapsCompatible_Level8/{z}/{y}/{x}.jpeg"],
            tileSize: 256,
            maxzoom: IMAGE_MAX_ZOOM,
          },
          [TERRAIN_SOURCE_ID]: {
            type: "raster-dem",
            tiles: ["https://elevation-tiles-prod.s3.amazonaws.com/terrarium/{z}/{x}/{y}.png"],
            tileSize: 256,
            maxzoom: 15,
            encoding: "terrarium",
          },
        },
        layers: [{
          id: "nasa-blue-marble-raster",
          type: "raster",
          source: NASA_SOURCE_ID,
          paint: { "raster-fade-duration": 250, "raster-resampling": "linear" },
        }],
      },
      center: DEFAULT_CENTER,
      zoom: DEFAULT_ZOOM,
      minZoom: 1.5,
      maxZoom: MAP_MAX_ZOOM,
      maxPitch: 60,
      pitch: 0,
      bearing: 0,
      projection: "mercator",
      attributionControl: { compact: true, customAttribution: attributionHTML() },
      dragRotate: true,
      touchPitch: true,
    });

    map.addControl(new maplibreApi.NavigationControl({ showZoom: false, showCompass: true, visualizePitch: true }), "top-right");
    for (const eventName of ["dragstart", "zoomstart", "rotatestart", "pitchstart"]) map.on(eventName, notifyInteract);
    map.on("error", (event) => {
      if (!active || !connected()) return;
      const error = event?.error;
      if (error?.name === "AbortError" || /abort/i.test(String(error?.message || ""))) return;
      if (event?.sourceId === NASA_SOURCE_ID) baseFailed = true;
      else if (event?.sourceId === TERRAIN_SOURCE_ID) terrainFailed = true;
      else otherResourcesFailed = true;
      statusForResources();
    });
    map.on("load", () => {
      if (!active || !connected() || !map) return;
      styleReady = true;
      try {
        map.addSource(ROUTE_SOURCE_ID, { type: "geojson", data: EMPTY_ROUTE });
        map.addLayer({
          id: ROUTE_LAYER_ID,
          type: "line",
          source: ROUTE_SOURCE_ID,
          layout: { "line-cap": "round", "line-join": "round" },
          paint: {
            "line-color": "#e6bd62",
            "line-width": ["interpolate", ["linear"], ["zoom"], 2, 2, 8, 3.4, 12, 4.4],
            "line-dasharray": [2.3, 1.35],
            "line-opacity": 0.96,
          },
        });
        apply3D(false);
        applyLatestState();
      } catch {
        otherResourcesFailed = true;
      }
      map.on("idle", () => { container.dataset.camera = JSON.stringify(readCamera()); statusForResources(); });
      statusForResources();
    });
    map.on("webglcontextlost", () => {
      otherResourcesFailed = true;
      reportStatus("degraded", "The map lost its WebGL drawing context. Reopen the map to try again.");
    });
    map.on("webglcontextrestored", () => {
      otherResourcesFailed = false;
      statusForResources();
    });
  } catch {
    if (active && connected()) {
      reportStatus("error", "The interactive map could not initialize. Reopen it to retry; the reader remains available.");
      destroy();
    }
  }

  return controller;
}
