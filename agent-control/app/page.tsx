"use client";

import { useCallback, useEffect, useRef, useState } from "react";

type Job = { id: string; issue: number; title: string; branch: string; status: string; container?: string; pr?: number; url?: string; error?: string; updatedAt: string };
type Status = { enabled: boolean; heartbeat: string | null; github: { login: string | null; error: string | null }; docker: { ok: boolean; error: string | null }; jobs: Job[]; events: { at: string; message: string }[]; config: { repo: string; concurrency: number }; queue: { number: number; title: string; reason?: string }[] };
const failed = (job: Job) => ["failed", "error"].includes(job.status);
const active = (job: Job) => ["starting", "running", "ci"].includes(job.status);
function time(value?: string | null) { if (!value) return "?"; const date = new Date(value); return Number.isNaN(date.getTime()) ? value : date.toLocaleString(); }
function safeUrl(value?: string) { try { const url = new URL(value || ""); return url.protocol === "https:" || url.protocol === "http:" ? url.href : undefined; } catch { return undefined; } }

export default function Dashboard() {
  const [data, setData] = useState<Status | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [actionError, setActionError] = useState("");
  const [actionNotice, setActionNotice] = useState("");
  const detailsRef = useRef<HTMLElement>(null);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const [pending, setPending] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const [section, setSection] = useState("overview");
  const alive = useRef(true);
  useEffect(() => {
    const sync = () => setSection(window.location.hash.slice(1) || "overview");
    sync(); window.addEventListener("hashchange", sync);
    return () => window.removeEventListener("hashchange", sync);
  }, []);
  useEffect(() => { if (selected) detailsRef.current?.focus(); }, [selected]);
  function closeDetails() { setSelected(null); triggerRef.current?.focus(); }
  const refresh = useCallback(async (signal?: AbortSignal) => {
    try {
      const response = await fetch("/api/status", { cache: "no-store", signal });
      if (!response.ok) throw new Error(`Status request failed (${response.status}).`);
      const next: Status = await response.json();
      if (alive.current && !signal?.aborted) { setData(next); setError(""); }
    } catch (e) { if (alive.current && !signal?.aborted) setError(e instanceof Error ? e.message : "Unable to reach Agent Control."); }
    finally { if (alive.current && !signal?.aborted) setLoading(false); }
  }, []);
  useEffect(() => {
    alive.current = true;
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout>;
    async function poll() { await refresh(controller.signal); if (!controller.signal.aborted) timer = setTimeout(poll, 4000); }
    void poll();
    return () => { alive.current = false; controller.abort(); clearTimeout(timer); };
  }, [refresh]);
  async function control(action: "start" | "pause" | "retry", id?: string) {
    setPending(id || action);
    setActionError(""); setActionNotice("");
    try {
      const response = await fetch("/api/control", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ action, ...(id ? { id } : {}) }) });
      if (!response.ok) { const body = await response.json().catch(() => null); throw new Error(body?.error || `Control request failed (${response.status}).`); }
      if (alive.current) { setActionNotice(`${action} request accepted.`); await refresh(); }
    } catch (e) { if (alive.current) setActionError(e instanceof Error ? e.message : "Action could not be completed."); }
    finally { if (alive.current) setPending(""); }
  }
  const job = data?.jobs.find(item => item.id === selected);
  const working = data?.jobs.filter(active).length ?? "?";
  return <div className="shell">
    <aside className="sidebar">
      <a className="brand" href="#overview" aria-label="Agent Control home" onClick={() => setSection("overview")}><span className="brand-mark">a<span>c</span></span><span>agent control<span className="brand-sub">AUTONOMOUS WORKSPACE</span></span></a>
      <div className="nav-label">WORKSPACE</div>
      <nav aria-label="Workspace">{[["overview", "◈", "Overview"], ["agents", "⌘", "Agents"], ["queue", "≡", "Issue queue"], ["activity", "↳", "Activity"]].map(([id, icon, label]) => <a key={id} className={section === id ? "nav-item selected" : "nav-item"} href={`#${id}`} aria-current={section === id ? "location" : undefined} onClick={() => setSection(id)}><span aria-hidden="true">{icon}</span>{label}{id === "agents" && <small>{data?.jobs.length ?? "—"}</small>}</a>)}</nav>
      <div className="sidebar-bottom"><span className="eyebrow">CONNECTED REPOSITORY</span><p className="repo">{data?.config.repo || "Awaiting configuration"}</p><span className="muted mono">Local execution · isolated workers</span></div>
    </aside>
    <div className="workspace">
      <header className="topbar"><div><span className="muted">Workspace</span><span className="slash">/</span>Overview</div><span className="mono connection"><i className={`dot ${data && !error ? "green" : ""}`} />{loading ? "Connecting" : error ? "Connection interrupted" : "Live connection"}</span></header>
      <main id="overview">
        <div className="page-heading"><div><div className="eyebrow orange">THE WORK, IN MOTION</div><h1>Mission control<span>.</span></h1><p className="muted">From open issue to pull request. One clear view of every agent.</p></div><div className="controls"><span className="mode mono"><i className={`dot ${data?.enabled ? "green" : ""}`} />{data ? data.enabled ? "RUNNING" : "PAUSED" : "?"}</span><button className="primary" disabled={!data || !!pending} onClick={() => void control(data?.enabled ? "pause" : "start")}>{pending === "start" || pending === "pause" ? "Updating…" : data?.enabled ? "Ⅱ Pause agents" : "▷ Start agents"}</button></div></div>
        {error && <div className="error-banner" role="alert"><span>{error} {data ? "Showing the last received state." : "Check that the local server is running."}</span><button disabled={!!pending} onClick={() => void refresh()}>Reconnect</button></div>}
        {actionError && <div className="error-banner action-error" role="alert"><span>Action failed: {actionError}</span><button onClick={() => setActionError("")}>Dismiss</button></div>}
        <p className="action-notice" role="status" aria-live="polite">{actionNotice}</p>
        {loading && <p className="loading" role="status">Connecting to your workspace…</p>}
        <section className="metrics" aria-label="Workspace metrics">{[["ACTIVE AGENTS", data ? working : "—", data ? `of ${data.config.concurrency} concurrent workers` : "Waiting for workspace"], ["IN THE QUEUE", data ? data.queue.length : "—", "Issues awaiting execution"], ["PULL REQUESTS", data ? data.jobs.filter(j => j.pr).length : "—", "Created by your agents"], ["NEEDS ATTENTION", data ? data.jobs.filter(failed).length : "—", "Jobs requiring a retry"]].map(([label, value, detail]) => <div className="metric" key={label}><span className="eyebrow">{label}</span><strong>{value}</strong><span className="muted">{detail}</span></div>)}</section>
        <section id="agents" className="section"><div className="section-heading"><h2>Agent fleet <span className="count">{data?.jobs.length ?? "—"}</span></h2><span className="muted mono">CAPACITY {data ? `${working} / ${data.config.concurrency}` : "—"}</span></div>
          <div className="fleet-ledger"><div className="ledger-heading mono" aria-hidden="true"><span>EXECUTION / ISSUE</span><span>LIFECYCLE</span><span>BRANCH / LAST SIGNAL</span><span>INSPECT</span></div>{data?.jobs.map(item => <button className={`fleet-row ${selected === item.id ? "chosen" : ""}`} key={item.id} onClick={event => { triggerRef.current = event.currentTarget; setSelected(item.id); if (selected === item.id) detailsRef.current?.focus(); }} aria-expanded={selected === item.id} aria-controls="agent-details"><span className="ledger-issue"><span className="mono orange">#{item.issue}</span><strong>{item.title}</strong><span className="mono muted">{item.id}</span></span><span className={`badge ${failed(item) ? "bad" : active(item) ? "live" : ""}`}>{item.status}</span><span className="ledger-signal"><span className="mono">? {item.branch || "?"}</span><time className="mono muted" dateTime={item.updatedAt}>{time(item.updatedAt)}</time></span><span className="inspect" aria-hidden="true">?</span></button>)}</div>
          {!loading && !data?.jobs.length && <div className="empty"><span className="empty-symbol" aria-hidden="true">⌘</span><h3>{data ? "Your fleet is standing by" : "Waiting for your workspace"}</h3><p>{data ? "Agents will appear here when an issue is assigned. Start agents to process the available queue." : "Agent status will appear when the server connects."}</p></div>}
        </section>
        <section id="agent-details" className="details" ref={detailsRef} tabIndex={-1} aria-labelledby="details-heading" hidden={!job}>{job && <><div className="section-heading"><h2 id="details-heading">Issue #{job.issue} <span className="muted">/ Agent details</span></h2><button aria-label="Close agent details" onClick={closeDetails}>×</button></div><h3>{job.title}</h3><dl>{[["Status", job.status], ["Branch", job.branch], ["Container", job.container || "Not assigned"], ["Updated", time(job.updatedAt)]].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value || "—"}</dd></div>)}</dl>{job.error && <p className="job-error" role="alert">{job.error}</p>}<div className="detail-actions">{safeUrl(job.url) && <a className="button" href={safeUrl(job.url)} target="_blank" rel="noreferrer">{job.pr ? `View pull request #${job.pr}` : "View job"} ↗</a>}{failed(job) && <button className="primary" disabled={!!pending} onClick={() => void control("retry", job.id)}>{pending === job.id ? "Retrying…" : "Retry job ↻"}</button>}</div></>}</section>
        <div className="lower-grid"><section id="queue" className="section"><div className="section-heading"><h2>Issue queue <span className="count">{data?.queue.length ?? "—"}</span></h2><span className="eyebrow">UP NEXT</span></div>{data?.queue.length ? <ol className="queue-list">{data.queue.map((item, index) => <li key={item.number}><span className="queue-position mono">{String(index + 1).padStart(2, "0")}</span><div><span className="mono orange">#{item.number}</span><h3>{item.title}</h3>{item.reason && <p className="muted">{item.reason}</p>}</div><span className="muted" aria-hidden="true">↗</span></li>)}</ol> : <div className="small-empty"><h3>{loading ? "Loading queue…" : "No issues queued"}</h3><p className="muted">{data ? "Eligible issues will appear here when available." : "Waiting for a successful status response."}</p></div>}</section>
        <section id="activity" className="section"><div className="section-heading"><h2>Activity log</h2><span className="mono muted">EVENT STREAM</span></div>{data?.events.length ? <ol className="activity-list">{data.events.map((event, index) => <li key={`${event.at}-${index}`}><i className="event-dot" /><div><time className="mono muted" dateTime={event.at}>{time(event.at)}</time><p>{event.message}</p></div></li>)}</ol> : <div className="small-empty"><h3>{loading ? "Loading events…" : "A quiet workspace"}</h3><p className="muted">{data ? "Agent events will appear here as work happens." : "Waiting for a successful status response."}</p></div>}</section></div>
        <section className="setup"><div><span className="eyebrow orange">CONNECTIONS</span><h2>Ready the launchpad.</h2><p className="muted">Authenticate on this machine in your terminal. Credentials stay outside the browser.</p><div className="setup-commands"><code>gh auth login</code><code>codex login</code></div></div><div className="connection-list"><div><span>GitHub</span><span className={data?.github.login ? "success-text" : "muted"}>{data?.github.login ? `@${data.github.login}` : !data ? loading ? "Checking?" : "?" : "Not connected"}</span></div>{data?.github.error && <p className="job-error">{data.github.error}</p>}<div><span>Docker</span><span className={data?.docker.ok ? "success-text" : "muted"}>{data ? data.docker.ok ? "Available" : "Unavailable" : loading ? "Checking?" : "?"}</span></div>{data?.docker.error && <p className="job-error">{data.docker.error}</p>}<p className="muted">Codex authentication is managed in your terminal.</p></div></section>
        <footer className="mono muted"><span><i className="dot" /> AGENT CONTROL</span><span>LAST HEARTBEAT · {time(data?.heartbeat)}</span><span>STATUS REFRESH · 4s</span></footer>
      </main>
    </div>
  </div>;
}
