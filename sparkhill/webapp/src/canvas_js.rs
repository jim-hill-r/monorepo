/// JavaScript for the canvas-based letter tracing component.
/// Matches the behavior of blue.eel.education's EelCanvas component.
pub const CANVAS_JS: &str = r#"
window.eelCanvas = (function() {
  // Guide line positions as fractions of canvas height (matching blue.eel.education)
  const CAP_RATIO    = 5  / 85;
  const MEAN_RATIO   = 30 / 85;
  const BASE_RATIO   = 55 / 85;
  const BEARD_RATIO  = 80 / 85;

  let canvas = null;
  let ctx    = null;
  let boundary = {};
  let drawing = false;
  let prevPoint = null;
  let recordedPoints = [];
  let isRecording = false;
  let isAnimating = false;
  let animationPoints = [];
  let animationSegment = 0;
  let timePerPoint = 0;
  let then = 0;
  let onAnimationComplete = null;

  function computeBoundary() {
    return {
      top:      0,
      capLine:   canvas.height * CAP_RATIO,
      meanLine:  canvas.height * MEAN_RATIO,
      baseLine:  canvas.height * BASE_RATIO,
      beardLine: canvas.height * BEARD_RATIO,
      bottom:    canvas.height
    };
  }

  function setStyle(type) {
    ctx.lineCap = 'round';
    ctx.strokeStyle = '#072A40';
    ctx.setLineDash([]);
    switch (type) {
      case 'USER':
        ctx.strokeStyle = '#178CA4'; ctx.lineWidth = 15; break;
      case 'EEL':
        ctx.strokeStyle = '#18B7BE'; ctx.lineWidth = 8; break;
      case 'START_CIRCLE':
        ctx.strokeStyle = 'white'; ctx.fillStyle = '#5A8100'; ctx.lineWidth = 2; break;
      case 'END_CIRCLE':
        ctx.strokeStyle = 'white'; ctx.fillStyle = '#B74803'; ctx.lineWidth = 2; break;
      case 'CAP_LINE': case 'BASE_LINE':
        ctx.lineWidth = 6; break;
      case 'MEAN_LINE':
        ctx.lineWidth = 3;
        ctx.setLineDash([canvas.width * 0.04, canvas.width * 0.02875]); break;
      case 'BEARD_LINE':
        ctx.lineWidth = 3; break;
    }
  }

  function paintLine(a, b) {
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
  }

  function paintCircle(p, r) {
    ctx.beginPath(); ctx.arc(p.x, p.y, r, 0, 2 * Math.PI, false);
    ctx.fill(); ctx.stroke();
  }

  function clearCanvas() {
    canvas.width  = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
    boundary = computeBoundary();
    canvas.style.background = '#F9F7F0';
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    setStyle('CAP_LINE');
    paintLine({x: 0, y: boundary.capLine},   {x: canvas.width, y: boundary.capLine});
    setStyle('BASE_LINE');
    paintLine({x: 0, y: boundary.baseLine},  {x: canvas.width, y: boundary.baseLine});
    setStyle('BEARD_LINE');
    paintLine({x: 0, y: boundary.beardLine}, {x: canvas.width, y: boundary.beardLine});
    setStyle('MEAN_LINE');
    paintLine({x: 0, y: boundary.meanLine},  {x: canvas.width, y: boundary.meanLine});
    setStyle('USER');
  }

  // --- Path math helpers ---
  function mag(v)      { return Math.sqrt(v.x*v.x + v.y*v.y); }
  function sub(a, b)   { return {x: a.x - b.x, y: a.y - b.y}; }
  function scale(p, s) { return {x: p.x*s, y: p.y*s, type: p.type}; }
  function translate(p, d) { return {x: p.x+d.x, y: p.y+d.y, type: p.type}; }
  function scalePath(path, s) { return path.map(p => scale(p, s)); }
  function translatePath(path, d) { return path.map(p => translate(p, d)); }

  function getBounds(path) {
    if (!path || path.length === 0) return {min:{x:0,y:0}, max:{x:0,y:0}, c:{x:0,y:0}, w:0, h:0};
    const mn = path.reduce((a, b) => ({x: Math.min(a.x,b.x), y: Math.min(a.y,b.y)}));
    const mx = path.reduce((a, b) => ({x: Math.max(a.x,b.x), y: Math.max(a.y,b.y)}));
    return {min:mn, max:mx, w:mx.x-mn.x, h:mx.y-mn.y, c:{x:0.5*(mn.x+mx.x), y:0.5*(mn.y+mx.y)}};
  }

  function normalizePattern(p) {
    const bnd = p.boundary;
    const s   = 100 / (bnd.baseLine - bnd.capLine);
    const scaled = scalePath(p.path, s);
    const bounds = getBounds(scaled);
    return { letter: p.letter, boundary: {top:bnd.top*s, capLine:bnd.capLine*s,
      meanLine:bnd.meanLine*s, baseLine:bnd.baseLine*s,
      beardLine:bnd.beardLine*s, bottom:bnd.bottom*s},
      path: translatePath(scaled, {x:-bounds.min.x, y:0}) };
  }

  function combinePatterns(patterns) {
    const gap = 15;
    const normed = patterns.map(p => normalizePattern(p));
    let combined = [];
    for (const n of normed) {
      const offset = combined.length ? getBounds(combined).max.x + gap : 0;
      combined = combined.concat(translatePath(n.path, {x:offset, y:0}));
    }
    return { boundary: normed[0].boundary, path: combined };
  }

  function fitToCanvas(combined) {
    const srcH = combined.boundary.baseLine - combined.boundary.capLine;
    const dstH = boundary.baseLine - boundary.capLine;
    const s    = dstH / srcH;
    const scaled = scalePath(combined.path, s);
    const bounds = getBounds(scaled);
    return translatePath(scaled, {x: canvas.width/2 - bounds.c.x, y: 0});
  }

  function pathLength(path) {
    let total = 0;
    for (let i = 0; i < path.length - 1; i++) total += mag(sub(path[i+1], path[i]));
    return total;
  }

  function downsample(path, target) {
    const res = path.slice();
    const step = res.length / (res.length - target);
    for (let i = res.length - 1; i > 0; i -= step) {
      const idx = Math.floor(i);
      if (res[idx] && res[idx].type !== 'start' && res[idx].type !== 'end' && res[idx].type !== 'critical') {
        res.splice(idx, 1);
      }
    }
    return res;
  }

  // --- Animation ---
  function animateStep() {
    if (!isAnimating) return;
    requestAnimationFrame(animateStep);
    const now = Date.now();
    if (now - then < timePerPoint) return;
    then = now;

    if (animationSegment >= animationPoints.length) {
      setStyle('USER');
      isAnimating = false;
      if (onAnimationComplete) onAnimationComplete();
      return;
    }
    const pt = animationPoints[animationSegment];
    if (pt.type === 'start') {
      setStyle('START_CIRCLE'); paintCircle(pt, 13);
    } else if (pt.type === 'end') {
      setStyle('END_CIRCLE'); paintCircle(pt, 13);
    } else {
      setStyle('EEL');
      const next = animationPoints[animationSegment + 1] || pt;
      paintLine(pt, next);
    }
    animationSegment++;
  }

  // --- Public API ---
  return {
    init(canvasId, animComplete) {
      canvas = document.getElementById(canvasId);
      if (!canvas) return false;
      ctx = canvas.getContext('2d');
      onAnimationComplete = animComplete;
      clearCanvas();
      // Set up mouse listeners
      canvas.addEventListener('mousedown',  e => {
        if (isAnimating) return;
        drawing = true;
        const r = canvas.getBoundingClientRect();
        const pt = {x: e.clientX - r.left, y: e.clientY - r.top, type: 'start'};
        prevPoint = pt;
        if (isRecording) recordedPoints.push(pt);
      });
      canvas.addEventListener('mousemove', e => {
        if (!drawing) return;
        const r = canvas.getBoundingClientRect();
        const pt = {x: e.clientX - r.left, y: e.clientY - r.top};
        paintLine(prevPoint, pt);
        if (isRecording) recordedPoints.push(pt);
        prevPoint = pt;
      });
      canvas.addEventListener('mouseup', () => {
        if (!drawing) return;
        if (prevPoint) { prevPoint.type = 'end'; if (isRecording) recordedPoints.push(prevPoint); }
        drawing = false;
      });
      canvas.addEventListener('mouseleave', () => { drawing = false; });
      // Touch listeners
      canvas.addEventListener('touchstart', e => {
        e.preventDefault();
        if (isAnimating) return;
        drawing = true;
        const r = canvas.getBoundingClientRect();
        const t = e.targetTouches[0];
        const pt = {x: t.clientX - r.left, y: t.clientY - r.top, type: 'start'};
        prevPoint = pt;
        if (isRecording) recordedPoints.push(pt);
      }, {passive: false});
      canvas.addEventListener('touchmove', e => {
        if (!drawing) return;
        const r = canvas.getBoundingClientRect();
        const t = e.targetTouches[0];
        const pt = {x: t.clientX - r.left, y: t.clientY - r.top};
        paintLine(prevPoint, pt);
        if (isRecording) recordedPoints.push(pt);
        prevPoint = pt;
      }, {passive: true});
      canvas.addEventListener('touchend',   () => { drawing = false; });
      canvas.addEventListener('touchcancel',() => { drawing = false; });
      return true;
    },

    clear() { if (canvas) { isAnimating = false; clearCanvas(); } },

    startRecording() { recordedPoints = []; isRecording = true; },

    stopRecording() {
      isRecording = false;
      return JSON.stringify({boundary, path: recordedPoints});
    },

    drawLetter(patternsJson, technique) {
      if (!canvas) return;
      const patterns = JSON.parse(patternsJson);
      const combined = combinePatterns(patterns);
      clearCanvas();
      const fitted = fitToCanvas(combined);
      if (!fitted || fitted.length === 0 || technique === 'Freeform') {
        setTimeout(() => { if (onAnimationComplete) onAnimationComplete(); }, 1200);
        return;
      }
      animationSegment = 0;
      animationPoints  = technique === 'Pattern'
        ? fitted.filter(p => p.type === 'start' || p.type === 'end')
        : fitted;
      timePerPoint = technique === 'Pattern' ? 400 : 3000 / animationPoints.length;
      then = Date.now();
      isAnimating = true;
      animateStep();
    },

    validateSuccess(patternsJson, recordingJson) {
      try {
        const patterns = JSON.parse(patternsJson);
        const recording = JSON.parse(recordingJson);
        if (!recording || recording.path.length < 10) return false;
        const template = combinePatterns(patterns.map(p => normalizePattern(p)));
        const user = recording.path;
        const tmpl = template.path;

        // Length check
        let t = tmpl.length > user.length ? downsample(tmpl, user.length) : tmpl;
        let u = user.length > tmpl.length ? downsample(user, tmpl.length) : user;
        const tLen = pathLength(t), uLen = pathLength(u);
        if (Math.abs(tLen - uLen) / (tLen || 1) > 0.125) return false;

        // Start/end point proximity check
        const check = (arr, type) => {
          const tPts = t.filter(p => p.type === type);
          const uPts = u.filter(p => p.type === type);
          const bnd  = getBounds(t);
          const sc   = Math.max(bnd.w, bnd.h) || 1;
          for (const tp of tPts) {
            const minD = uPts.reduce((m, up) => Math.min(m, mag(sub(tp, up))), sc);
            if (minD / sc > 0.1) return false;
          }
          return true;
        };
        if (!check(t, 'start') || !check(u, 'end')) return false;

        // Bounding-box similarity
        const tb = getBounds(t), ub = getBounds(u);
        const sc = Math.max(tb.w, tb.h) || 1;
        if (Math.abs(tb.w - ub.w) / sc > 0.125) return false;
        if (Math.abs(tb.h - ub.h) / sc > 0.125) return false;
        if (Math.abs(tb.c.x - ub.c.x) / sc > 0.1)  return false;
        if (Math.abs(tb.c.y - ub.c.y) / sc > 0.1)  return false;

        return true;
      } catch(_) { return false; }
    }
  };
})();
"#;
