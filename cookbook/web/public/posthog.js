// PostHog session replay for the cookbook's production site (decision 0001,
// standards/docs/plans/2026-11-deploy-feedback.md, Phase 4).
//
// Served as a file from 'self' so the CSP needs no 'unsafe-inline'. It loads
// PostHog only on the production host, never on previews or localhost, never
// for our own synthetic traffic (the jimhillr-evals user agent, or a browser
// driven by automation), and never when the visitor asks not to be tracked.
//
// The project token (phc_) is PostHog's public, ingest-only key: it ships in
// every page by design, so it is config here, not a secret. Personal keys
// (phx_) are secrets and never go in this file (.gitleaks.toml flags them).
(function () {
  "use strict";

  // The project token, from PostHog > Settings > Project > General.
  var PROJECT = "phc_CraDKPCKB7kBc3WiwDhDCfj3S5nD8vha3xtDBVQzS8Ak";
  // US Cloud. For EU, use https://eu.i.posthog.com and
  // https://eu-assets.i.posthog.com, here and in _headers and src/main.rs.
  var API_HOST = "https://us.i.posthog.com";
  var ASSETS_HOST = "https://us-assets.i.posthog.com";
  var PRODUCTION_HOST = "jimhillr-cookbook.pages.dev";

  if (window.location.hostname !== PRODUCTION_HOST) return;
  if (/jimhillr-evals/i.test(navigator.userAgent) || navigator.webdriver) return;
  if (
    navigator.doNotTrack === "1" ||
    window.doNotTrack === "1" ||
    navigator.globalPrivacyControl === true
  ) {
    return;
  }

  // Drops the Auth0 sign-in code and state from captured URLs. auth_sdk
  // removes them from the address bar after sign-in, but a page view can be
  // captured first; PKCE already makes a leaked code useless.
  function redactUrl(value) {
    if (typeof value !== "string" || value.indexOf("?") < 0) return value;
    try {
      var url = new URL(value);
      url.searchParams.delete("code");
      url.searchParams.delete("state");
      return url.toString();
    } catch (e) {
      return value;
    }
  }

  var options = {
    api_host: API_HOST,
    person_profiles: "identified_only",
    persistence: "localStorage",
    capture_pageview: "history_change",
    capture_pageleave: true,
    capture_dead_clicks: true,
    capture_exceptions: true,
    respect_dnt: true,
    disable_surveys: true,
    enable_recording_console_log: true,
    capture_performance: { network_timing: true, web_vitals: false },
    session_recording: {
      maskAllInputs: true,
      maskTextSelector: "[data-ph-mask]",
      recordHeaders: false,
      recordBody: false,
    },
    before_send: function (event) {
      if (event && event.properties) {
        event.properties.$current_url = redactUrl(event.properties.$current_url);
        event.properties.$referrer = redactUrl(event.properties.$referrer);
      }
      return event;
    },
  };

  var script = document.createElement("script");
  script.src = ASSETS_HOST + "/static/array.js";
  script.async = true;
  script.crossOrigin = "anonymous";
  script.onload = function () {
    if (window.posthog && typeof window.posthog.init === "function") {
      window.posthog.init(PROJECT, options);
    }
  };
  document.head.appendChild(script);
})();
