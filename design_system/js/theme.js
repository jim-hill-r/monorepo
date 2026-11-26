/* design_system theme switcher. Inline it in the page <head>, before any
 * stylesheet, so the saved choice applies before first paint:
 *
 *   <html data-ds-default="professional" data-ds-themes="earthy airy professional colorful gamey">
 *   <script>...this file...</script>
 *
 * data-ds-default is the app's default theme, data-ds-themes the themes it
 * offers (optional). The choice is kept in localStorage under "ds-theme" as
 * {"theme": "...", "scheme": "system|light|dark", "a11y": "system|on|off"}.
 * "system" leaves the attribute off, so the page follows
 * prefers-color-scheme and prefers-contrast. Exposes window.dsTheme.get()
 * and window.dsTheme.set(choice). */
(function () {
  var KEY = "ds-theme";
  var root = document.documentElement;

  function read() {
    try {
      var c = JSON.parse(window.localStorage.getItem(KEY) || "{}");
      return c && typeof c === "object" ? c : {};
    } catch (e) {
      return {};
    }
  }

  function offered(theme) {
    var list = (root.getAttribute("data-ds-themes") || "").split(/\s+/);
    return !!theme && (list[0] === "" || list.indexOf(theme) >= 0);
  }

  function apply(c) {
    var fallback = root.getAttribute("data-ds-default") || root.getAttribute("data-theme") || "professional";
    root.setAttribute("data-theme", offered(c.theme) ? c.theme : fallback);
    if (c.scheme === "light" || c.scheme === "dark") {
      root.setAttribute("data-scheme", c.scheme);
    } else {
      root.removeAttribute("data-scheme");
    }
    if (c.a11y === "on" || c.a11y === "off") {
      root.setAttribute("data-a11y", c.a11y);
    } else {
      root.removeAttribute("data-a11y");
    }
  }

  window.dsTheme = {
    get: function () {
      var c = read();
      return {
        theme: root.getAttribute("data-theme"),
        scheme: c.scheme === "light" || c.scheme === "dark" ? c.scheme : "system",
        a11y: c.a11y === "on" || c.a11y === "off" ? c.a11y : "system"
      };
    },
    set: function (c) {
      try {
        window.localStorage.setItem(KEY, JSON.stringify(c));
      } catch (e) {
        /* No storage: the choice lasts until the page reloads. */
      }
      apply(c);
    }
  };

  apply(read());
})();
