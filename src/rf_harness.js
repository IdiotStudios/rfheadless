var __rfox_dom = @ELEMENTS@;

function __matches_simple(el, sel) {
    if (!sel) return false;
    if (sel[0] === '#') return el.id === sel.slice(1);
    if (sel[0] === '.') return (el.class||'').split(/\s+/).indexOf(sel.slice(1)) !== -1;
    // attribute selectors with optional operators: = ~= ^= $= *= |=
    var attrm = sel.match(/^\[([^~|\^\$\*=\]]+)(?:([~\^\$\*\|]?=)\s*(?:['\"]?([^'\"]+)['\"]?)\s*)?\]$/);
    if (attrm) {
        var name = attrm[1].trim();
        var op = attrm[2];
        var val = attrm[3];
        var aval = el.getAttribute(name);
        if (op === undefined) return aval !== null;
        if (aval === null) return false;
        if (op === '=') return aval === val;
        if (op === '~=') return (aval.split(/\s+/).indexOf(val) !== -1);
        if (op === '^=') return aval.indexOf(val) === 0;
        if (op === '$=') return aval.indexOf(val, aval.length - val.length) !== -1;
        if (op === '*=') return aval.indexOf(val) !== -1;
        if (op === '|=') return (aval === val || aval.indexOf(val + '-') === 0);
        return false;
    }
    // pseudo-classes: :first-child, :last-child
    var pc = sel.match(/:([a-z-]+)$/);
    var pcname = null;
    if (pc) {
        pcname = pc[1];
        sel = sel.replace(/:[a-z-]+$/, '');
        sel = sel.trim();
    }
    if (pcname) {
        var idx = __rfox_dom.indexOf(el);
        if (idx === -1) { return false; }
        var parent = el.parent;
        var siblings = [];
        for (var si=0; si<__rfox_dom.length; si++) { if (__rfox_dom[si].parent === parent) siblings.push(si); }
        if (pcname === 'first-child') {
            if (siblings.length === 0) return false;
            if (siblings[0] !== idx) return false;
        }
        if (pcname === 'last-child') {
            if (siblings.length === 0) return false;
            if (siblings[siblings.length-1] !== idx) return false;
        }
    }
    var parts = sel.split('.');
    if (parts.length === 2) return el.tag.toLowerCase() === parts[0].toLowerCase() && (el.class||'').split(/\s+/).indexOf(parts[1]) !== -1;
    return el.tag.toLowerCase() === sel.toLowerCase();
}

function __matches(el, selector) {
    selector = selector.trim();
    // Fast path for simple selectors
    if (selector.indexOf(' ') === -1 && selector.indexOf('>') === -1) {
        return __matches_simple(el, selector);
    }
    // Complex selector: right-to-left matching
    var tokens = selector.split(/\s+/);
    var cur = el;
    var i = tokens.length - 1;
    while (i >= 0) {
        var token = tokens[i];
        if (token === '>') {
            i--;
            if (i < 0) return false;
            var sel = tokens[i];
            var pidx = cur.parent;
            if (pidx === null || pidx === undefined) return false;
            var parent = __rfox_dom[pidx];
            if (!__matches_simple(parent, sel)) return false;
            cur = parent;
            i--;
            continue;
        } else {
            var sel = token;
            // For descendant combinator, find an ancestor matching sel
            var pidx = cur.parent;
            var found = false;
            // Right-most token may refer to the element itself
            if (i === tokens.length - 1 && __matches_simple(cur, sel)) { i--; continue; }
            while (pidx !== null && pidx !== undefined) {
                var ancestor = __rfox_dom[pidx];
                if (__matches_simple(ancestor, sel)) { found = true; cur = ancestor; break; }
                pidx = ancestor.parent;
            }
            if (!found) return false;
            i--;
            continue;
        }
    }
    return true;
}

function querySelector(sel) {
    var parts = sel.split(',').map(function(s){return s.trim();});
    for (var i=0;i<__rfox_dom.length;i++) {
        for (var pi=0; pi<parts.length; pi++) {
            if (__matches(__rfox_dom[i], parts[pi])) return __wrap_el(__rfox_dom[i]);
        }
    }
    return __wrap_el(null);
}

function querySelectorAll(sel) {
    var out = [];
    var parts = sel.split(',').map(function(s){return s.trim();});
    for (var i=0;i<__rfox_dom.length;i++) {
        for (var pi=0; pi<parts.length; pi++) {
            if (__matches(__rfox_dom[i], parts[pi])) out.push(__wrap_el(__rfox_dom[i]));
        }
    }
    return out;
}

// Wrap element with safe helpers to avoid TypeErrors.
function __wrap_el(el) {
    if (!el) {
        return { text: "", id: "", class: "", tag: "", attributes: [], getAttribute: function() { return null; }, textContent: function() { return ""; }, innerHTML: function(v) { if (arguments.length) { this.text = v; } return (this.text === undefined || this.text === null) ? "" : this.text; } };
    }
    if (!el.getAttribute) {
        el.getAttribute = function(n) {
            for (var i=0;i<this.attributes.length;i++) { if (this.attributes[i][0] === n) return this.attributes[i][1]; }
            return null;
        };
    }
    if (!el.setAttribute) {
        el.setAttribute = function(n, v) {
            for (var i=0;i<this.attributes.length;i++) { if (this.attributes[i][0] === n) { this.attributes[i][1] = String(v); return; } }
            this.attributes.push([n, String(v)]);
            // keep dataset in sync if data-* attribute
            if (n.indexOf('data-') === 0 && this.dataset) {
                var name = n.slice(5).replace(/-([a-z])/g,function(m,p){return p.toUpperCase();});
                try { this.dataset[name] = String(v); } catch(e) {}
            }
        };
    }
    if (!el.textContent) {
        el.textContent = function() { return (this.text === undefined || this.text === null) ? "" : this.text; };
    }
    // dataset: expose data-* attributes as camelCase props and helpers
    if (!el.dataset) {
        el.dataset = (function(e) { var out = {}; for (var i=0;i<e.attributes.length;i++) { var k=e.attributes[i][0]; if (k.indexOf('data-')===0) { var name = k.slice(5).replace(/-([a-z])/g,function(m,p){return p.toUpperCase();}); out[name]=e.attributes[i][1]; } } out.get = function(n) { return out[n] || null; }; out.set = function(n, v) { try { e.setAttribute('data-' + n.replace(/([A-Z])/g, function(m,p){return '-' + p.toLowerCase();}), String(v)); out[n] = String(v); } catch(e) {} }; return out; })(el);
    }
    // classList helper with add/remove/toggle/contains and helpers
    if (!el.classList) {
        el.classList = (function(e) {
            return {
                add: function(c) { var parts = (e.class||'').split(/\s+/).filter(Boolean); if (parts.indexOf(c)===-1) { parts.push(c); e.class=parts.join(' '); e.setAttribute('class', e.class); } },
                remove: function(c) { var parts = (e.class||'').split(/\s+/).filter(Boolean); e.class = parts.filter(function(p){return p!==c;}).join(' '); e.setAttribute('class', e.class); },
                toggle: function(c) { if (this.contains(c)) this.remove(c); else this.add(c); },
                contains: function(c) { return (e.class||'').split(/\s+/).indexOf(c)!==-1; },
                length: function() { return (e.class||'').split(/\s+/).filter(Boolean).length; },
                toString: function() { return (e.class||'').trim(); }
            };
        })(el);
    }
    if (!el.querySelector) {
        el.querySelector = function(sel) { for (var i=0;i<__rfox_dom.length;i++) { if (__matches(__rfox_dom[i], sel)) return __wrap_el(__rfox_dom[i]); } return __wrap_el(null); };
    }
    if (!el.querySelectorAll) {
        el.querySelectorAll = function(sel) { var out=[]; for (var i=0;i<__rfox_dom.length;i++) { if (__matches(__rfox_dom[i], sel)) out.push(__wrap_el(__rfox_dom[i])); } return out; };
    }
    return el;
}

function querySelector(sel) {
    for (var i=0;i<__rfox_dom.length;i++) { if (__matches(__rfox_dom[i], sel)) return __wrap_el(__rfox_dom[i]); }
    return __wrap_el(null);
}

function querySelectorAll(sel) {
    var out = [];
    for (var i=0;i<__rfox_dom.length;i++) { if (__matches(__rfox_dom[i], sel)) out.push(__wrap_el(__rfox_dom[i])); }
    return out;
}

// Simple CSS parser: populate __rfox_rules from document.styles
var __rfox_styles = @STYLES@;
var __rfox_rules = [];
(function() {
    var rule_re = /([^\{]+)\{([^\}]+)\}/g;
    function computeSpec(sel) {
        var idc = (sel.match(/#[\w-]+/g) || []).length;
        var clc = ((sel.match(/\.[\w-]+/g) || []).length) + ((sel.match(/\[[^\]]+\]/g) || []).length);
        var tagc = (sel.replace(/#[\w-]+/g, '').replace(/\.[\w-]+/g, '').replace(/\[[^\]]+\]/g, '').trim().match(/[a-zA-Z][\w-]*/g) || []).length;
        return idc * 10000 + clc * 100 + tagc;
    }
    for (var si=0; si<__rfox_styles.length; si++) {
        var s = __rfox_styles[si];
        var m;
        while ((m = rule_re.exec(s)) !== null) {
            var sels = m[1].trim().split(',');
            var decls = {};
            m[2].split(';').forEach(function(d) {
                var parts = d.split(':');
                if (parts.length === 2) { decls[parts[0].trim()] = parts[1].trim(); }
            });
            sels.forEach(function(subsel) {
                var rs = subsel.trim();
                var spec = computeSpec(rs);
                __rfox_rules.push({ selector: rs, decls: decls, specificity: spec, order: __rfox_rules.length });
            });
        }
    }
})();

// Normalization helpers for computed styles (extended)
function normalizeColor(val) {
    if (!val) return val;
    val = String(val).trim().toLowerCase();
    var hexm = val.match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/);
    if (hexm) {
        var h = hexm[1];
        if (h.length === 3) {
            h = h.split('').map(function(c){return c+c}).join('');
        }
        return '#' + h;
    }
    var rgbm = val.match(/^rgba?\(([^)]+)\)$/);
    if (rgbm) {
        var parts = rgbm[1].split(',').map(function(p){return p.trim();});
        var r = parseInt(parts[0])||0;
        var g = parseInt(parts[1])||0;
        var b = parseInt(parts[2])||0;
        var a = parts[3] !== undefined ? parseFloat(parts[3]) : 1;
        if (a >= 1) {
            return '#' + [r,g,b].map(function(n){ return ('0'+(n&255).toString(16)).slice(-2); }).join('');
        } else {
            return 'rgba(' + [r,g,b,a].join(',') + ')';
        }
    }
    // hsl/hsla -> rgb -> hex/rgba
    var hslm = val.match(/^hsla?\(([^)]+)\)$/);
    if (hslm) {
        var parts = hslm[1].split(',').map(function(p){return p.trim();});
        var h = parseFloat(parts[0]) || 0;
        var s = (parts[1]||'0').replace('%',''); s = parseFloat(s)/100 || 0;
        var l = (parts[2]||'0').replace('%',''); l = parseFloat(l)/100 || 0;
        var a = parts[3] !== undefined ? parseFloat(parts[3]) : 1;
        function hue2rgb(p, q, t) {
            if (t < 0) t += 1;
            if (t > 1) t -= 1;
            if (t < 1/6) return p + (q - p) * 6 * t;
            if (t < 1/2) return q;
            if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
            return p;
        }
        var r,g,b;
        if (s === 0) { r = g = b = l; }
        else {
            var q = l < 0.5 ? l * (1 + s) : l + s - l * s;
            var p = 2 * l - q;
            var hk = (h % 360) / 360;
            r = hue2rgb(p,q,hk + 1/3);
            g = hue2rgb(p,q,hk);
            b = hue2rgb(p,q,hk - 1/3);
        }
        var R = Math.round(r * 255), G = Math.round(g * 255), B = Math.round(b * 255);
        if (a >= 1) return '#' + [R,G,B].map(function(n){ return ('0'+(n&255).toString(16)).slice(-2); }).join('');
        return 'rgba(' + [R,G,B,a].join(',') + ')';
    }
    var named = {'red':'#ff0000','green':'#008000','blue':'#0000ff','black':'#000000','white':'#ffffff'};
    if (named[val]) return named[val];
    return val;
}

function normalizeUnit(val) {
    if (val === undefined || val === null) return val;
    var s = String(val).trim().toLowerCase();
    if (/^[0-9.]+$/.test(s)) return s + 'px';
    // strip spaces and normalize like "12 PX" -> "12px"
    s = s.replace(/\s+/g,'');
    return s;
}

// getComputedStyle that applies rules by specificity & order, with inline style winning
// Normalizes colors and common unit properties
function getComputedStyle(el) {
    if (!el || !el.getAttribute) return { getPropertyValue: function() { return ''; } };
    var matched = [];
    for (var i=0;i<__rfox_rules.length;i++) {
        var r = __rfox_rules[i];
        try {
            if (__matches(el, r.selector)) matched.push(r);
        } catch(e) { /* ignore selector errors */ }
    }
    matched.sort(function(a,b) {
        if (a.specificity !== b.specificity) return a.specificity - b.specificity;
        return a.order - b.order;
    });
    var decls = {};
    for (var j=0;j<matched.length;j++) {
        var d = matched[j].decls;
        for (var k in d) { if (Object.prototype.hasOwnProperty.call(d,k)) decls[k.toLowerCase()] = d[k]; }
    }
    // inline style overrides
    var styleAttr = el.getAttribute('style') || '';
    styleAttr.split(';').forEach(function(s) { var p = s.split(':'); if (p.length === 2) decls[p[0].trim().toLowerCase()] = p[1].trim(); });

    return {
        getPropertyValue: function(prop) {
            var key = prop.toLowerCase();
            var v = decls[key];
            if (v === undefined) return '';
            if (key.indexOf('color') !== -1 || key === 'background') return normalizeColor(v);
            var unitProps = ['font-size','margin','margin-top','margin-bottom','padding','padding-top','padding-bottom','width','height'];
            if (unitProps.indexOf(key) !== -1) return normalizeUnit(v);
            return String(v).trim();
        }
    };
} 

var __rfox_console = [];
var document = { title: @TITLE@, body: @BODY@, styles: @STYLES@, querySelector: querySelector, querySelectorAll: querySelectorAll };
var console = { log: function() { var txt = Array.prototype.slice.call(arguments).join(' '); var st=''; try{ st=(new Error()).stack || (new Error()).toString(); }catch(e){} if (typeof __rfox_console_log === 'function') { try{ __rfox_console_log(txt, st); }catch(e){} } else { __rfox_console.push(txt); } }, error: function() { var txt = Array.prototype.slice.call(arguments).join(' '); var st=''; try{ st=(new Error()).stack || (new Error()).toString(); }catch(e){} if (typeof __rfox_console_error === 'function') { try{ __rfox_console_error(txt, st); }catch(e){} } else { __rfox_console.push(txt); } } };