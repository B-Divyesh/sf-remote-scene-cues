<script lang="ts">
  import { onMount } from 'svelte';
  import { csvEscape, formatClock, parseCues, type ParsedCue } from './lib';

  type Cue = ParsedCue & { id: string; position: number };
  type CueEvent = { id: string; sequence: number; cue_id: string; scene: string; cue_name: string; controller_name: string; webhook_status: string; created_at: string };
  type Controller = { id: string; name: string; status: string; created_at: string };
  type Snapshot = { code: string; title: string; role: string; status: string; current_index: number; cues: Cue[]; controllers: Controller[]; events: CueEvent[]; expires_at: string; webhook_configured: boolean };

  const defaultCues = `Arrival | Open house lights
Arrival | Start lobby music
Briefing | Fade house lights
Briefing | Show rules camera
Act one | Open room
Act one | Reveal first clue
Act two | Switch to overhead camera
Act two | Trigger warning sound
Finale | Unlock final door
Finale | Full lights and applause`;
  const pathParts = location.pathname.split('/').filter(Boolean);
  let page = pathParts[0] || 'home';
  let code = (pathParts[1] || '').toUpperCase();
  let online = navigator.onLine;
  let busy = false;
  let error = '';
  let notice = '';
  let snapshot: Snapshot | null = null;
  let qrData = '';
  let roomToken = code ? sessionStorage.getItem(`scene-cues:${page}:${code}`) || '' : '';
  let storedJoinUrl = code ? sessionStorage.getItem(`scene-cues:join-url:${code}`) || '' : '';
  let joinSecret = new URLSearchParams(location.search).get('secret') || '';
  let controllerName = '';
  let showName = 'Friday rehearsal';
  let cuesText = defaultCues;
  let webhookEnabled = false;
  let webhookUrl = '';
  let webhookSecret = '';
  let eventSource: EventSource | null = null;

  onMount(() => {
    if (joinSecret && roomToken) {
      history.replaceState({}, '', `/join/${code}`);
      joinSecret = '';
    }
    const up = () => { online = true; notice = 'Back online — syncing the room.'; if (roomToken && code) loadRoom(); };
    const down = () => { online = false; };
    addEventListener('online', up); addEventListener('offline', down);
    if (code && roomToken) startRoom();
    return () => { removeEventListener('online', up); removeEventListener('offline', down); eventSource?.close(); };
  });

  async function api<T>(url: string, options: RequestInit = {}): Promise<T> {
    if (!navigator.onLine) throw new Error('You are offline. Reconnect to continue.');
    const response = await fetch(url, { ...options, headers: { 'content-type': 'application/json', ...(options.headers || {}) } });
    if (!response.ok) {
      if (response.status === 429) throw new Error('Too many requests reached the room at once. Wait a moment, then try again.');
      if (response.status === 413) throw new Error('That cue sheet is too large. Keep it to 12 short cues and shorten long cue text.');
      const data = await response.json().catch(() => ({ error: 'The room could not be reached.' }));
      throw new Error(data.error || 'The request failed.');
    }
    return response.status === 204 ? (undefined as T) : response.json();
  }

  async function createRoom() {
    error = ''; notice = '';
    const cues = parseCues(cuesText);
    const max = 12;
    if (!showName.trim()) { error = 'Give this rehearsal a name.'; return; }
    if (!cues.length) { error = 'Add at least one cue.'; return; }
    if (cues.length > max) { error = `A room can hold up to ${max} cues. Shorten this sheet, then try again.`; return; }
    busy = true;
    try {
      const room = await api<{code:string;host_token:string;join_url:string}>('/api/rooms', { method: 'POST', body: JSON.stringify({ title: showName, cues, webhook_url: webhookEnabled ? webhookUrl : null, webhook_secret: webhookEnabled ? webhookSecret : null }) });
      sessionStorage.setItem(`scene-cues:host:${room.code}`, room.host_token);
      sessionStorage.setItem(`scene-cues:join-url:${room.code}`, room.join_url);
      sessionStorage.setItem('scene-cues:last-host', room.code);
      location.href = `/host/${room.code}`;
    } catch (e) { error = (e as Error).message; } finally { busy = false; }
  }

  async function joinRoom() {
    if (!joinSecret) { error = 'This join link is incomplete. Ask the host for a fresh QR code.'; return; }
    if (!controllerName.trim()) { error = 'Name this controller so the host knows whom to approve.'; return; }
    busy = true; error = '';
    try {
      const joined = await api<{token:string}>(`/api/rooms/${code}/join`, { method: 'POST', body: JSON.stringify({ secret: joinSecret, name: controllerName }) });
      roomToken = joined.token;
      sessionStorage.setItem(`scene-cues:join:${code}`, roomToken);
      history.replaceState({}, '', `/join/${code}`);
      joinSecret = '';
      await startRoom();
    } catch (e) { error = (e as Error).message; } finally { busy = false; }
  }

  async function startRoom() {
    await loadRoom();
    eventSource?.close();
    eventSource = new EventSource(`/api/rooms/${code}/events?token=${encodeURIComponent(roomToken)}`);
    eventSource.addEventListener('refresh', () => loadRoom());
    eventSource.onerror = () => { if (online) notice = 'Live link interrupted — reconnecting automatically.'; };
  }

  async function loadRoom() {
    try {
      snapshot = await api<Snapshot>(`/api/rooms/${code}?token=${encodeURIComponent(roomToken)}`);
      error = ''; notice = '';
      if (page === 'host' && snapshot) {
        if (storedJoinUrl) {
          const QRCode = (await import('qrcode')).default;
          qrData = await QRCode.toDataURL(storedJoinUrl, { width: 320, margin: 1, color: { dark: '#151512', light: '#fbf9f1' } });
        }
      }
    } catch (e) {
      error = (e as Error).message;
      if (error.includes('expired') || error.includes('not found')) snapshot = null;
    }
  }

  async function roomAction(path: string, method = 'POST', body: object | undefined = undefined) {
    busy = true; error = '';
    try {
      await api(`/api/rooms/${code}${path}?token=${encodeURIComponent(roomToken)}`, { method, body: body ? JSON.stringify(body) : undefined });
      await loadRoom();
    } catch (e) { error = (e as Error).message; } finally { busy = false; }
  }

  async function fire() {
    if (!snapshot || snapshot.status !== 'approved') return;
    await roomAction('/fire');
    if (!error) notice = `Receipt ${snapshot?.events[0]?.sequence || ''}: cue acknowledged by the room.`;
  }

  function keyboardGo(event: KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (['INPUT','TEXTAREA','SELECT'].includes(target.tagName)) return;
    if (event.key.toLowerCase() === 'g' && !busy) { event.preventDefault(); fire(); return; }
    if (page !== 'host' || !snapshot || !['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) return;
    const cueButtons = Array.from(document.querySelectorAll<HTMLButtonElement>('.cue-list button:not(:disabled)'));
    if (!cueButtons.length) return;
    const current = cueButtons.indexOf(document.activeElement as HTMLButtonElement);
    const direction = event.key === 'ArrowUp' || event.key === 'ArrowLeft' ? -1 : 1;
    const fallback = Math.max(0, Math.min(cueButtons.length - 1, snapshot.current_index + direction));
    const next = current < 0 ? fallback : (current + direction + cueButtons.length) % cueButtons.length;
    event.preventDefault();
    cueButtons[next].focus();
  }

  function copyJoin() {
    navigator.clipboard.writeText(storedJoinUrl).then(() => notice = 'Controller link copied.').catch(() => error = 'Copy failed. Select the link manually.');
  }

  function downloadLog() {
    if (!snapshot) return;
    const rows = [['Sequence','Time','Scene','Cue','Controller','Webhook'], ...[...snapshot.events].reverse().map((e) => [e.sequence,e.created_at,e.scene,e.cue_name,e.controller_name,e.webhook_status])];
    const csv = rows.map((row) => row.map(csvEscape).join(',')).join('\n');
    const link = document.createElement('a'); link.href = URL.createObjectURL(new Blob([csv], { type: 'text/csv' })); link.download = `${snapshot.code}-cue-log.csv`; link.click(); URL.revokeObjectURL(link.href);
  }

  async function endRoom() {
    if (!confirm(`End ${snapshot?.title}? Its controllers and cue log will be deleted now.`)) return;
    await roomAction('', 'DELETE');
    if (!error) { sessionStorage.removeItem(`scene-cues:host:${code}`); location.href = '/'; }
  }

  async function clearLog() {
    if (confirm('Clear this room’s cue receipt log? This cannot be undone.')) await roomAction('/logs', 'DELETE');
  }

</script>

<svelte:window onkeydown={keyboardGo} />

<a class="skip-link" href="#main">Skip to main content</a>
<div class:offline={!online} class="network" role="status" aria-live="polite">{online ? 'Live link ready' : 'Offline — controls paused'}</div>

<header class="masthead">
  <a class="wordmark" href="/" aria-label="Scene Cues home"><span aria-hidden="true">SC</span> Scene Cues</a>
  <nav aria-label="Primary">
    <a href="/#how">How it works</a>
  </nav>
</header>

{#if page === 'privacy'}
  <main id="main" class="legal">
    <p class="kicker">Legal / Privacy</p><h1>Short rooms.<br/>Short memory.</h1>
    <p>Scene Cues stores room names, cue text, controller names, webhook settings, and cue receipts on our server only while a room is live. Rooms and their logs expire after eight hours, or immediately when the host ends the room. We do not use analytics, advertising cookies, or third-party trackers.</p>
    <h2>On your device</h2><p>Room access tokens live in session storage and disappear when the browser session closes. Scene Cues does not store room data in local storage. You can remove any browser site data with your browser’s site-data controls.</p>
    <h2>Webhooks</h2><p>If a host configures a webhook, cue payloads are sent only to that HTTPS address and signed with the supplied secret. Private and local-network addresses are rejected.</p>
    <p>Questions: <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a></p>
  </main>
{:else if page === 'terms'}
  <main id="main" class="legal">
    <p class="kicker">Legal / Terms</p><h1>A rehearsal aid,<br/>not a safety system.</h1>
    <p>Scene Cues is provided “as is” for creative rehearsals and performances. Do not use it for pyrotechnics, life-safety systems, access control, or any action where a late, duplicated, or unavailable network message could cause harm.</p>
    <h2>Rooms and acceptable use</h2><p>You are responsible for cue content, controller approvals, webhook destinations, and keeping room links private. Do not use the service to attack systems or send unlawful material. We may limit abusive traffic.</p>
  </main>
{:else if page === 'join'}
  <main id="main" class="controller-main">
    {#if !roomToken}
      <section class="join-sheet">
        <p class="folio">Controller / Room {code || 'unknown'}</p>
        <h1>Ask to join<br/>the cue desk.</h1>
        <p class="lede">The host will see this name and approve this device before it can fire anything.</p>
        <form onsubmit={(e) => { e.preventDefault(); joinRoom(); }}>
          <label for="controller-name">Controller name</label>
          <input id="controller-name" bind:value={controllerName} autocomplete="nickname" maxlength="40" placeholder="e.g. Sam — booth" />
          <button class="ink-button" disabled={busy}>{busy ? 'Requesting…' : 'Request host approval'}</button>
        </form>
      </section>
    {:else if snapshot?.status === 'pending'}
      <section class="waiting"><p class="folio">Room {code}</p><h1>Waiting at<br/>the door.</h1><div class="stamp">Approval pending</div><p>Keep this page open. The controls will appear as soon as the host approves this device.</p></section>
    {:else if snapshot?.status === 'rejected'}
      <section class="waiting"><p class="folio">Room {code}</p><h1>Not approved.</h1><p>The host declined this controller. Ask them for a fresh link if that was a mistake.</p></section>
    {:else if snapshot}
      <section class="remote" aria-labelledby="remote-title">
        <div class="remote-head"><p class="folio">{snapshot.title} / {snapshot.code}</p><p>{snapshot.events.length} receipts</p></div>
        <p class="kicker">On now</p>
        <h1 id="remote-title">{snapshot.current_index >= 0 ? snapshot.cues[snapshot.current_index]?.name : 'Stand by'}</h1>
        <p class="scene-name">{snapshot.current_index >= 0 ? snapshot.cues[snapshot.current_index]?.scene : 'No cue fired yet'}</p>
        <button class="go-button" onclick={fire} disabled={busy || snapshot.current_index >= snapshot.cues.length - 1 || !online} aria-describedby="next-cue">
          <span>{busy ? 'Sending' : snapshot.current_index >= snapshot.cues.length - 1 ? 'Complete' : 'GO'}</span>
          <small>G key</small>
        </button>
        <div id="next-cue" class="next-strip"><span>Next</span><strong>{snapshot.cues[snapshot.current_index + 1]?.name || 'End of cue sheet'}</strong></div>
        {#if snapshot.events[0]}<p class="receipt" aria-live="polite">✓ Receipt {snapshot.events[0].sequence} · {formatClock(snapshot.events[0].created_at)}</p>{/if}
      </section>
    {:else}
      <section class="waiting"><p class="folio">Room {code}</p><h1>Room<br/>unavailable.</h1><p>{error || 'Checking the live room…'}</p><a class="ink-button link-button" href="/">Return home</a></section>
    {/if}
    {#if error}<p class="alert error" role="alert">{error}</p>{/if}
    {#if notice}<p class="alert" role="status">{notice}</p>{/if}
  </main>
{:else if page === 'host'}
  <main id="main" class="host-main">
    {#if !roomToken}
      <section class="waiting"><p class="folio">Room {code}</p><h1>Host key<br/>not found.</h1><p>This room’s host key only lives in the browser session that created it. Return to that tab, or start a fresh room.</p><a class="ink-button link-button" href="/">Start a new room</a></section>
    {:else if !snapshot}
      <section class="waiting"><p class="folio">Room {code}</p><h1>{error ? 'Room unavailable.' : 'Opening the cue desk…'}</h1><p>{error || 'Checking the live room…'}</p>{#if error}<a class="ink-button link-button" href="/">Start a fresh room</a>{/if}</section>
    {:else}
      <section class="host-hero">
        <div><p class="folio">Live host / Room {snapshot.code}</p><h1>{snapshot.title}</h1><p class="expires">Auto-deletes {new Date(snapshot.expires_at).toLocaleString()}</p></div>
        <div class="join-card">
          {#if qrData}<img src={qrData} width="160" height="160" alt="QR code for controllers to request access to room {code}" />{/if}
          <div><p class="kicker">Controller entry</p>{#if storedJoinUrl}<a class="join-link" href={storedJoinUrl}>{storedJoinUrl}</a>{:else}<p>Reopen the original creation tab to retrieve the private join link.</p>{/if}<button class="text-button" onclick={copyJoin} disabled={!storedJoinUrl}>Copy private link</button></div>
        </div>
      </section>

      {#if snapshot.controllers.some((c) => c.status === 'pending')}
        <section class="approvals" aria-labelledby="approval-title"><p class="kicker">Needs your say</p><h2 id="approval-title">Controller requests</h2>
          {#each snapshot.controllers.filter((c) => c.status === 'pending') as controller}
            <div class="approval-row"><strong>{controller.name}</strong><div><button onclick={() => roomAction(`/controllers/${controller.id}/approve`)}>Approve</button><button class="quiet" onclick={() => roomAction(`/controllers/${controller.id}/reject`)}>Decline</button></div></div>
          {/each}
        </section>
      {/if}

      <section class="desk-grid">
        <div class="cue-desk">
          <div class="section-heading"><div><p class="kicker">Cue sheet</p><h2>{snapshot.current_index + 1} / {snapshot.cues.length}</h2></div><button class="go-compact" onclick={fire} disabled={busy || snapshot.current_index >= snapshot.cues.length - 1 || !online}>GO <span>G</span></button></div>
          <ol class="cue-list">
            {#each snapshot.cues as cue}
              <li class:current={cue.position === snapshot.current_index} class:done={cue.position < snapshot.current_index}>
                <button onclick={() => roomAction('/set', 'POST', { index: cue.position })} disabled={busy} aria-current={cue.position === snapshot.current_index ? 'step' : undefined}>
                  <span class="cue-no">{String(cue.position + 1).padStart(2, '0')}</span><span><small>{cue.scene}</small><strong>{cue.name}</strong></span><span class="state-word">{cue.position === snapshot.current_index ? 'LIVE' : cue.position < snapshot.current_index ? 'DONE' : 'SET'}</span>
                </button>
              </li>
            {/each}
          </ol>
        </div>
        <aside class="room-rail">
          <p class="kicker">Approved controllers</p>
          {#if snapshot.controllers.filter((c) => c.status === 'approved').length}
            <ul class="controller-list">{#each snapshot.controllers.filter((c) => c.status === 'approved') as c}<li><span aria-hidden="true">●</span> {c.name}</li>{/each}</ul>
          {:else}<p class="empty-copy">No controllers yet. Have a teammate scan the code above.</p>{/if}
          <p class="kicker rail-kicker">Webhook</p><p>{snapshot.webhook_configured ? 'Signed delivery is on.' : 'Not configured for this room.'}</p>
          <p class="micro">Every cue advances on the server before delivery. A failed webhook never changes cue order.</p>
        </aside>
      </section>

      <section class="log-section">
        <div class="section-heading"><div><p class="kicker">Receipts</p><h2>Cue log</h2></div><div class="button-row"><button class="text-button" onclick={downloadLog} disabled={!snapshot.events.length}>Export CSV</button><button class="text-button danger-text" onclick={clearLog} disabled={!snapshot.events.length}>Clear log</button></div></div>
        {#if snapshot.events.length}
          <div class="table-wrap"><table><thead><tr><th>#</th><th>Time</th><th>Cue</th><th>Controller</th><th>Webhook</th></tr></thead><tbody>{#each snapshot.events as item}<tr><td>{item.sequence}</td><td>{formatClock(item.created_at)}</td><td><small>{item.scene}</small>{item.cue_name}</td><td>{item.controller_name}</td><td>{item.webhook_status.replace('_', ' ')}</td></tr>{/each}</tbody></table></div>
        {:else}<div class="empty-log"><strong>No receipts yet.</strong><p>Your first GO will appear here with its server time and controller name.</p></div>{/if}
      </section>
      <section class="danger-zone"><div><h2>Close the room</h2><p>Ending now deletes controller access and the server-side cue log.</p></div><button class="outline-danger" onclick={endRoom}>End and delete room</button></section>
    {/if}
    {#if error}<p class="alert error" role="alert">{error}</p>{/if}
    {#if notice}<p class="alert" role="status">{notice}</p>{/if}
  </main>
{:else}
  <main id="main">
    <section class="hero">
      <div class="hero-copy"><p class="folio">Vol. 01 / Live rehearsal utility</p><h1>Advance the scene.<br/><em>Skip the lobby.</em></h1><p class="lede">A phone-friendly cue desk for tiny live-game, escape-room, and theatre teams. Share one private QR. Approve the crew. Fire every cue in order.</p><a class="ink-button link-button" href="#create">Open a rehearsal room <span aria-hidden="true">↓</span></a></div>
      <figure><picture><source type="image/avif" media="(min-width: 900px)" srcset="/assets/scene-cues-hero-large.avif"/><source type="image/avif" srcset="/assets/scene-cues-hero-mobile.avif"/><source media="(min-width: 900px)" srcset="/assets/scene-cues-hero-large.webp"/><img src="/assets/scene-cues-hero.webp" width="960" height="640" alt="Woodcut-style cue sheets, stage lights, coiled cable, and one red cue button" fetchpriority="high" decoding="async" /></picture><figcaption>One cue light. One shared truth. Original AI-assisted illustration.</figcaption></figure>
    </section>
    <section class="proof-strip" aria-label="Product facts"><span>01 / Eight-hour rooms</span><span>02 / Host-approved devices</span><span>03 / Signed webhook receipts</span></section>

    <section id="create" class="create-section">
      <div class="create-intro"><p class="kicker">Make the call sheet</p><h2>Room setup</h2><p>No account. Nothing to install. Room data and receipts delete automatically after eight hours.</p></div>
      <form class="setup-form" onsubmit={(e) => { e.preventDefault(); createRoom(); }}>
        <label for="show-name">Rehearsal name</label><input id="show-name" bind:value={showName} maxlength="80" required />
        <div class="label-row"><label for="cues">Scenes and cues</label><span>{parseCues(cuesText).length} / 12</span></div>
        <textarea id="cues" bind:value={cuesText} rows="11" spellcheck="true" aria-describedby="cue-help"></textarea><p id="cue-help" class="field-help">One per line: <strong>Scene | Cue name</strong></p>
        <details><summary>Signed webhook <span>optional</span></summary><div class="details-body"><label class="check-row"><input type="checkbox" bind:checked={webhookEnabled}/> Send every cue to an existing app</label>{#if webhookEnabled}<label for="webhook-url">Public HTTPS endpoint</label><input id="webhook-url" type="url" bind:value={webhookUrl} placeholder="https://example.com/cues" required/><label for="webhook-secret">Signing secret</label><input id="webhook-secret" bind:value={webhookSecret} minlength="12" autocomplete="new-password" required/><p class="field-help">Sent as HMAC-SHA256 in <code>X-Scene-Cues-Signature</code>. Local network targets are blocked.</p>{/if}</div></details>
        <button class="go-create" disabled={busy}>{busy ? 'Opening room…' : 'Create room + private QR'}</button>
        {#if error}<p class="form-error" role="alert">{error}</p>{/if}{#if notice}<p class="form-notice" role="status">{notice}</p>{/if}
      </form>
    </section>

    <section id="how" class="how-section"><p class="kicker">The running order</p><h2>From call sheet to GO<br/>in under a minute.</h2><ol><li><span>1</span><div><strong>Write the sequence</strong><p>Name the scenes and cues in their real running order.</p></div></li><li><span>2</span><div><strong>Approve the booth</strong><p>Controllers scan the private QR; nothing works until the host says yes.</p></div></li><li><span>3</span><div><strong>Call GO</strong><p>Every press advances once on the server, logs a receipt, and optionally posts a signed webhook.</p></div></li></ol></section>

  </main>
{/if}

<footer><p>Scene Cues — made for the moment before “GO.”</p><nav aria-label="Legal"><a href="/privacy">Privacy</a><a href="/terms">Terms</a><a href="https://sociobot.in">A Param Factory product</a></nav><p class="micro">Hero imagery is original and AI-assisted; provenance is documented in the source.</p></footer>
