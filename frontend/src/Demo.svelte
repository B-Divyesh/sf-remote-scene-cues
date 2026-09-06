<script lang="ts">
  import { onMount } from 'svelte';
  import { csvEscape, formatClock } from './lib';

  type DemoCue = { scene: string; name: string };
  type DemoReceipt = { sequence: number; scene: string; cue: string; controller: string; at: string };
  type DemoState = { current: number; approved: boolean; cueCount: number; receipts: DemoReceipt[] };

  const storageKey = 'demo:scene-cues:sample-v1';
  const cues: DemoCue[] = [
    { scene: 'Arrival', name: 'Open house lights' },
    { scene: 'Arrival', name: 'Start lobby music' },
    { scene: 'Briefing', name: 'Fade house lights' },
    { scene: 'Briefing', name: 'Show rules camera' },
    { scene: 'Act one', name: 'Open room' },
    { scene: 'Act one', name: 'Reveal first clue' },
    { scene: 'Act two', name: 'Switch to overhead camera' },
    { scene: 'Act two', name: 'Trigger warning sound' },
    { scene: 'Finale', name: 'Unlock final door' },
    { scene: 'Finale', name: 'Full lights and applause' }
  ];

  function baseline(): DemoState {
    return {
      current: 2,
      approved: false,
      cueCount: cues.length,
      receipts: [
        { sequence: 3, scene: 'Briefing', cue: 'Fade house lights', controller: 'Mina — lighting', at: '19:04:12' },
        { sequence: 2, scene: 'Arrival', cue: 'Start lobby music', controller: 'Mina — lighting', at: '19:03:49' },
        { sequence: 1, scene: 'Arrival', cue: 'Open house lights', controller: 'Mina — lighting', at: '19:03:18' }
      ]
    };
  }

  function readState(): DemoState {
    try {
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        const parsed = JSON.parse(stored) as DemoState;
        if (Array.isArray(parsed.receipts) && Number.isInteger(parsed.current)) return parsed;
      }
    } catch {
      // A blocked storage area should still leave a complete in-memory demo.
    }
    return baseline();
  }

  let state = baseline();
  let workspaceId = '';
  let notice = '';

  function persist() {
    try { localStorage.setItem(storageKey, JSON.stringify(state)); } catch { /* demo remains in memory */ }
  }

  async function provision() {
    try {
      const response = await fetch('/api/demo/workspaces', { method: 'POST', headers: { 'content-type': 'application/json' } });
      if (response.ok) workspaceId = (await response.json()).id || '';
    } catch {
      // Offline reloads use the sample already stored in the demo namespace.
    }
  }

  onMount(() => {
    state = readState();
    provision();
  });

  function approveSampleController() {
    state = { ...state, approved: true };
    persist();
    notice = 'Alex — props is approved and can now send GO.';
  }

  function fire() {
    if (!state.approved || state.current >= cues.length - 1) return;
    const next = state.current + 1;
    const cue = cues[next];
    const receipt: DemoReceipt = {
      sequence: (state.receipts[0]?.sequence || 0) + 1,
      scene: cue.scene,
      cue: cue.name,
      controller: 'Alex — props',
      at: formatClock(new Date().toISOString())
    };
    state = { ...state, current: next, receipts: [receipt, ...state.receipts] };
    persist();
    notice = `Receipt ${receipt.sequence}: ${cue.name} was acknowledged.`;
  }

  function addCue() {
    if (state.cueCount >= 12) {
      notice = 'A room can hold up to 12 cues.';
      return;
    }
    state = { ...state, cueCount: state.cueCount + 1 };
    persist();
    notice = `${state.cueCount} of 12 sample cues are ready.`;
  }

  function clearLog() {
    state = { ...state, receipts: [] };
    persist();
    notice = 'The sample receipt log is clear.';
  }

  function resetDemo() {
    state = baseline();
    try { localStorage.removeItem(storageKey); } catch { /* nothing else to clear */ }
    persist();
    notice = 'The sample room is reset.';
  }

  function startForReal() {
    try { localStorage.removeItem(storageKey); } catch { /* nothing else to clear */ }
    if (workspaceId) fetch(`/api/demo/workspaces/${encodeURIComponent(workspaceId)}`, { method: 'DELETE' }).catch(() => undefined);
  }

  function downloadLog() {
    const rows = [['Sequence', 'Time', 'Scene', 'Cue', 'Controller'], ...[...state.receipts].reverse().map((receipt) => [receipt.sequence, receipt.at, receipt.scene, receipt.cue, receipt.controller])];
    const csv = rows.map((row) => row.map(csvEscape).join(',')).join('\n');
    const link = document.createElement('a');
    link.href = URL.createObjectURL(new Blob([csv], { type: 'text/csv' }));
    link.download = 'scene-cues-sample-log.csv';
    link.click();
    URL.revokeObjectURL(link.href);
  }

  function keyboardGo(event: KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (['INPUT', 'TEXTAREA', 'SELECT', 'BUTTON', 'A'].includes(target.tagName)) return;
    if (event.key.toLowerCase() === 'g') { event.preventDefault(); fire(); }
  }
</script>

<svelte:window onkeydown={keyboardGo} />

<main id="main" class="demo-main">
  <aside class="demo-banner" aria-label="Demo status" role="status">
    <strong>Demo — sample data, nothing is saved</strong>
    <div><button class="text-button" onclick={resetDemo}>Reset demo</button><a class="text-button" href="/" onclick={startForReal}>Start for real</a></div>
  </aside>

  <section class="demo-intro">
    <p class="folio">Sample rehearsal / Room LANTERN</p>
    <h1>Run shared rehearsal cues from approved phones</h1>
    <p class="lede">Try a ten-cue escape-room tech rehearsal. Approve Alex, then send the next cue.</p>
  </section>

  <section class="demo-workspace" aria-labelledby="sample-room-title">
    <div class="demo-current">
      <p class="kicker">Live sample room</p>
      <h2 id="sample-room-title">The Lantern Room<br/>tech rehearsal</h2>
      <p class="expires">Sample room data is separate from real rehearsal rooms.</p>
      <div class="sample-state"><span>On now</span><strong>{cues[state.current].name}</strong><small>{cues[state.current].scene}</small></div>
      <button class="go-create demo-go" onclick={fire} disabled={!state.approved || state.current >= cues.length - 1} aria-describedby="demo-go-help">{state.current >= cues.length - 1 ? 'Cue sheet complete' : 'Send GO for next cue'}</button>
      <p id="demo-go-help" class="field-help">{state.approved ? 'Alex — props is approved.' : 'Approve Alex before this phone can send GO.'}</p>
    </div>

    <aside class="demo-approval" aria-labelledby="demo-approval-title">
      <p class="kicker">Controller approval</p>
      <h2 id="demo-approval-title">Phone requests</h2>
      <div class="sample-controller"><strong>Mina — lighting</strong><span>Approved</span></div>
      <div class="sample-controller"><strong>Alex — props</strong>{#if state.approved}<span>Approved</span>{:else}<button onclick={approveSampleController}>Approve Alex — props</button>{/if}</div>
      <p class="field-help">The host decides which phone can advance a real room.</p>
    </aside>
  </section>

  <section class="demo-log" aria-labelledby="demo-log-title">
    <div class="section-heading"><div><p class="kicker">Sample output</p><h2 id="demo-log-title">Cue receipts</h2></div><div class="button-row"><button class="text-button" onclick={downloadLog} disabled={!state.receipts.length}>Download sample CSV</button><button class="text-button danger-text" onclick={clearLog} disabled={!state.receipts.length}>Clear sample log</button></div></div>
    {#if state.receipts.length}
      <div class="table-wrap"><table><thead><tr><th>Receipt</th><th>Time</th><th>Cue</th><th>Controller</th></tr></thead><tbody>{#each state.receipts as receipt}<tr><td>{receipt.sequence}</td><td>{receipt.at}</td><td><small>{receipt.scene}</small>{receipt.cue}</td><td>{receipt.controller}</td></tr>{/each}</tbody></table></div>
    {:else}
      <div class="empty-log"><strong>No sample receipts remain.</strong><p>Reset the demo to restore the populated sample log.</p></div>
    {/if}
  </section>

  <section class="demo-limit" aria-labelledby="demo-limit-title">
    <div><p class="kicker">Cue limit</p><h2 id="demo-limit-title">{state.cueCount} of 12 sample cues</h2><p>Small rehearsal rooms support up to 12 ordered cues.</p></div><button class="outline-danger" onclick={addCue}>Add sample cue</button>
  </section>
  {#if notice}<p class="alert" role="status">{notice}</p>{/if}
</main>
