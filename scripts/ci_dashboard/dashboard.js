/* Standalone dashboard: no CDN, tracking, network requests or untrusted HTML. */
"use strict";
const history = JSON.parse(document.getElementById("history-data").textContent);
const names = {
  nightly: "Nightly verification",
  benchmarks: "Weekly CPU benchmarks",
  macos: "macOS build",
  windows: "Windows build",
};
const runs = history.runs;
function el(tag, text, cls) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (cls) node.className = cls;
  return node;
}
function link(text, url) {
  const node = el("a", text);
  if (/^https:\/\/github\.com\//.test(url)) {
    node.href = url;
    node.rel = "noopener";
  }
  return node;
}
function when(row) {
  return new Date(row.created_at).toISOString().slice(0, 16).replace("T", " ");
}
function badge(status) {
  return el("span", status, "badge " + status);
}
const latest = runs[0];
if (latest) {
  document.getElementById("updated").textContent =
    "Updated " + when(latest) + " UTC";
  const base = latest.url.split("/actions/")[0];
  document
    .getElementById("links")
    .append(
      link("Repository", base),
      link("Actions reports", base + "/actions"),
      link("Latest run + artifacts", latest.url),
    );
}
for (const [lane, name] of Object.entries(names)) {
  const row = runs.find((r) => r.lane === lane);
  const card = el("article", undefined, "card");
  card.append(el("h3", name), badge(row ? row.status : "not run"));
  if (row) {
    card.append(
      el("p", when(row) + " UTC"),
      el("p", row.revision.slice(0, 8)),
      link("Inspect run →", row.url),
    );
    const failed = row.checks.filter((c) => c.status !== "success");
    if (failed.length)
      card.append(
        el("p", "Attention: " + failed.map((c) => c.name).join(", ")),
      );
  } else card.append(el("p", "Waiting for the first scheduled or manual run."));
  document.getElementById("cards").append(card);
}
function table() {
  const filter = document.getElementById("lane").value;
  const selected = runs.filter((r) => filter === "all" || r.lane === filter);
  const body = document.getElementById("runs");
  body.replaceChildren();
  for (const row of selected) {
    const tr = el("tr");
    const status = el("td");
    status.append(badge(row.status));
    const revision = el("td");
    revision.append(el("code", row.revision.slice(0, 8)));
    const details = el("td");
    details.append(link("Logs / artifacts ↗", row.url));
    tr.append(
      el("td", when(row)),
      el("td", names[row.lane]),
      status,
      revision,
      el(
        "td",
        row.checks.filter((c) => c.status === "success").length +
          "/" +
          row.checks.length +
          " passed",
      ),
      details,
    );
    body.append(tr);
  }
  document.getElementById("history-empty").hidden = selected.length > 0;
}
document.getElementById("lane").addEventListener("change", table);
table();
const groups = new Map();
for (const row of [...runs].reverse()) {
  for (const m of row.metrics || []) {
    const key = m.comparison_key;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push({ row, m });
  }
}
const selector = document.getElementById("metric");
for (const [key, points] of groups) {
  const m = points[0].m;
  const option = el(
    "option",
    m.name + " · " + m.context + " · " + points[0].row.environment.cpu,
  );
  option.value = key;
  selector.append(option);
}
const NS = "http://www.w3.org/2000/svg";
function svg(tag, attrs, text) {
  const node = document.createElementNS(NS, tag);
  for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, String(v));
  if (text !== undefined) node.textContent = text;
  return node;
}
function chart() {
  const target = document.getElementById("chart");
  target.replaceChildren();
  const points = groups.get(selector.value) || [];
  const empty = document.getElementById("chart-empty");
  empty.hidden = points.length > 0;
  target.style.display = points.length ? "block" : "none";
  if (!points.length) {
    empty.textContent =
      "No benchmark measurements yet. Weekly CPU results will appear here; missing or failed suites never create zero-valued samples.";
    selector.disabled = true;
    return;
  }
  const max = Math.max(...points.map((p) => p.m.value), 0.001) * 1.15;
  const width = Math.max(220, target.getBoundingClientRect().width);
  target.setAttribute("viewBox", `0 0 ${width} 240`);
  const left = 58,
    right = width - 15,
    top = 18,
    bottom = 195;
  const x = (i) =>
    points.length === 1
      ? (left + right) / 2
      : left + ((right - left) * i) / (points.length - 1);
  const y = (v) => bottom - ((bottom - top) * v) / max;
  for (let i = 0; i <= 4; i++) {
    const value = (max * i) / 4;
    target.append(
      svg("line", {
        x1: left,
        x2: right,
        y1: y(value),
        y2: y(value),
        stroke: "#263344",
      }),
      svg(
        "text",
        {
          x: left - 10,
          y: y(value) + 4,
          fill: "#96a7ba",
          "font-size": 11,
          "text-anchor": "end",
        },
        value.toFixed(2),
      ),
    );
  }
  target.append(
    svg("polyline", {
      points: points.map((p, i) => `${x(i)},${y(p.m.value)}`).join(" "),
      fill: "none",
      stroke: "#6fe2c1",
      "stroke-width": 2,
    }),
  );
  for (let i = 0; i < points.length; i++) {
    const p = points[i];
    const circle = svg("circle", {
      cx: x(i),
      cy: y(p.m.value),
      r: 4,
      fill: p.row.status === "success" ? "#6fe2c1" : p.row.status === "pending" ? "#f3d479" : "#ff9a9a",
    });
    circle.append(
      svg(
        "title",
        {},
        `${when(p.row)} UTC · ${p.row.revision.slice(0, 8)} · ${p.m.value.toFixed(3)} ms · ${p.row.status}`,
      ),
    );
    const anchor = svg(
      "a",
      /^https:\/\/github\.com\//.test(p.row.url) ? { href: p.row.url } : {},
    );
    anchor.append(circle);
    target.append(anchor);
  }
  for (const i of new Set([0, points.length - 1]))
    target.append(
      svg(
        "text",
        {
          x: x(i),
          y: bottom + 25,
          fill: "#96a7ba",
          "font-size": 11,
          "text-anchor": "middle",
        },
        when(points[i].row).slice(0, 10),
      ),
    );
  document.getElementById("chart-details").classList.remove("regression");
  const last = points.at(-1),
    previous = points.at(-2);
  let detail = `${points.length} comparable sample(s) · latest ${last.m.value.toFixed(3)} ms · ${last.row.revision.slice(0, 8)}`;
  if (previous && previous.m.value > 0) {
    const delta = 100 * (last.m.value / previous.m.value - 1);
    detail += ` · ${delta >= 0 ? "+" : ""}${delta.toFixed(1)}% versus previous (positive = slower)`;
    if (delta > 20 && previous.m.value >= 0.1) detail += " · Review: potential regression (>20% increase)";
    document.getElementById("chart-details").classList.toggle("regression", delta > 20 && previous.m.value >= 0.1);
  }
  document.getElementById("chart-details").textContent = detail;
}
selector.addEventListener("change", chart);
window.addEventListener("resize", chart);
chart();
