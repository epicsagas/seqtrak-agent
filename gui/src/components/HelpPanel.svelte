<script lang="ts">
  // In-GUI manual: per-view control reference with English default and Korean toggle.
  let { view, onclose }: { view: 'device' | 'mixer'; onclose: () => void } = $props()
  let lang = $state<'en' | 'ko'>('en')

  function stop(e: Event) {
    e.stopPropagation()
  }
</script>

<div class="backdrop" role="presentation" onclick={onclose} onkeydown={(e) => e.key === 'Escape' && onclose()}>
  <div class="panel" role="dialog" tabindex="-1" aria-label={`${view} manual`} onclick={stop} onkeydown={stop}>
    <header>
      <h2>{view === 'device' ? 'DEVICE MANUAL' : 'MIXER MANUAL'}</h2>
      <div class="actions">
        <button class="lbtn" class:on={lang === 'en'} onclick={() => (lang = 'en')}>EN</button>
        <button class="lbtn" class:on={lang === 'ko'} onclick={() => (lang = 'ko')}>KO</button>
        <button class="x" aria-label="close manual" onclick={onclose}>CLOSE</button>
      </div>
    </header>
    <div class="body">
      {#if lang === 'en'}
        <p class="flow"><b>Note:</b> This web GUI is currently a prototype and some controls may not behave accurately yet. For reliable control, use the <b>st</b> CLI or <b>st-mcp</b>.</p>
        {#if view === 'device'}
          <p class="flow">Select tracks and patterns with the top knobs, toggle 16th-note steps on the drum grid, record scale notes with REC while playing, and shape the sound in the FX section.</p>
          <h3>SIDE BUTTONS</h3>
          <ul>
            <li><b>VOL+ / VOL-</b> Monitor level for the unit speaker and headphones. Sends no MIDI.</li>
            <li><b>MUTE / SOLO</b> Arm mute or solo mode, then click a track knob to apply. Click again to exit.</li>
            <li><b>PG1 / PG2</b> Switch the drum step grid between steps 1-16 and 17-32.</li>
            <li><b>PLAY</b> Start or stop playback. Follows the hardware PLAY button automatically.</li>
            <li><b>DELETE</b> Hardware-only delete function; displays a status hint.</li>
          </ul>
          <h3>ALL KNOB</h3>
          <ul>
            <li>Turn to switch patterns 1-6 across all 11 tracks at once (sends SysEx Pattern Select to the unit).</li>
            <li>Click to send All Notes Off (panic).</li>
          </ul>
          <h3>TRACK KNOBS (KICK TO SAMPLER)</h3>
          <ul>
            <li>Turn to select pattern 1-6 for that track on the hardware.</li>
            <li>Click to select the track (and audition when stopped). Selecting a track reads its current pattern number from the unit.</li>
            <li>Hold to mute the track.</li>
          </ul>
          <h3>DRUM STEP GRID (2x8)</h3>
          <ul>
            <li>Toggle 16th-note steps for the selected track. Lit cells are active steps; the dark border marks the playhead.</li>
            <li>Playback locks onto the unit's 96 PPQ MIDI clock and follows the hardware PLAY button.</li>
            <li><em>Steps live in the GUI layer.</em> To write notes into the unit's own pattern memory, arm REC on the hardware and run <b>st rec-steps</b> from the CLI or MCP.</li>
          </ul>
          <h3>REC + C4 TO B4</h3>
          <ul>
            <li>Arm <b>REC</b> (orange) and press scale keys while playing to record into the current step.</li>
            <li>If the hardware REC button is also armed, those same Note On messages are recorded into the unit's pattern.</li>
            <li>Pressing scale keys while REC is off auditions the sound without recording.</li>
          </ul>
          <h3>SOUND DESIGN &amp; FX</h3>
          <ul>
            <li><b>4 SD knobs</b> Adjust CUTOFF, RESO, ATTACK, and DECAY for the selected track (CC 74/71/73/75).</li>
            <li><b>FX knob</b> Switch effect type (category x preset via SysEx).</li>
            <li><b>MST / SINGLE</b> Latching slide switch for effect target scope.</li>
            <li><b>Touch sliders</b> Page 1: FX LEVEL, plus momentary HIGH PASS and RPT strips that spring back to 0 on release. Page 2: assigned FX parameters 1, 2, and 3.</li>
          </ul>
        {:else}
          <p class="flow">The mixer view puts all 11 track strips, drum trigger pads, arpeggiator controls, transport, and project loading on a single screen.</p>
          <h3>TRACK STRIPS</h3>
          <ul>
            <li><b>volume / pan</b> Sliders send CC 7 and CC 10 on each track channel.</li>
            <li><b>M</b> Mute, <b>S</b> Solo, <b>HIT</b> Audition root note.</li>
          </ul>
          <h3>DRUMS ZONE</h3>
          <ul>
            <li><b>7 pads</b> Trigger the 7 drum tracks (channels 1-7) immediately.</li>
          </ul>
          <h3>SYNTH / SAMPLER ZONE</h3>
          <ul>
            <li><b>ARP block</b> Arpeggiator template, gate, speed, mono/poly/chord mode, and portamento for synth1, synth2, or dx.</li>
          </ul>
          <h3>MASTER ZONE</h3>
          <ul>
            <li><b>ALL knob</b> Broadcasts CC 7 volume across all 11 tracks.</li>
            <li><b>TRANSPORT</b> Start or stop the sequencer.</li>
            <li><b>PROJECT P1-P8</b> Load projects 1 to 8 on the unit (auto-saves current buffer on switch).</li>
            <li><b>IN log</b> Displays recent incoming MIDI messages from the unit.</li>
          </ul>
          <h3>RECORDING INTO HARDWARE MEMORY</h3>
          <ul>
            <li>To record step patterns into the unit's own memory, arm REC on the hardware panel and run <b>st rec-steps</b> (chain tracks with <code>--keep-playing</code>).</li>
          </ul>
        {/if}
      {:else}
        <p class="flow"><b>참고:</b> 이 웹 GUI는 현재 프로토타입 상태이므로 일부 조작이 정확히 동작하지 않을 수 있습니다. 안정적인 제어는 <b>st</b> CLI 또는 <b>st-mcp</b>를 사용하세요.</p>
        {#if view === 'device'}
          <p class="flow">상단 노브로 트랙과 패턴을 고르고, 드럼 그리드로 16분음표 스텝을 찍고, 재생 중 REC와 스케일 키로 실시간 입력한 뒤 FX 섹션으로 다듬습니다.</p>
          <h3>사이드 버튼</h3>
          <ul>
            <li><b>VOL+ / VOL-</b> 본체 스피커/헤드폰 모니터 음량입니다. MIDI 메시지는 나가지 않습니다.</li>
            <li><b>MUTE / SOLO</b> 모드를 켠 뒤 트랙 노브를 누르면 해당 트랙에 적용됩니다. 다시 누르면 해제됩니다.</li>
            <li><b>PG1 / PG2</b> 드럼 스텝 그리드의 1-16스텝과 17-32스텝 페이지를 전환합니다.</li>
            <li><b>PLAY</b> 재생 및 정지. 장비 본체의 PLAY 버튼 상태와 자동으로 동기화됩니다.</li>
            <li><b>DELETE</b> 본체 전용 삭제 기능으로 화면에 안내 문구만 표시합니다.</li>
          </ul>
          <h3>ALL 노브</h3>
          <ul>
            <li>돌리면 11개 전체 트랙의 패턴(1-6)을 동시에 변경합니다(실제 장비로 SysEx Pattern Select 전송).</li>
            <li>클릭하면 전체 노트 오프(panic)를 보냅니다.</li>
          </ul>
          <h3>트랙 노브 (KICK ~ SAMPLER)</h3>
          <ul>
            <li>돌리면 해당 트랙의 패턴(1-6)을 실제로 변경합니다.</li>
            <li>클릭하면 트랙을 선택하며(정지 중에는 청음), 장비에서 현재 패턴 번호를 읽어와 동기화합니다.</li>
            <li>길게 누르면 해당 트랙을 뮤트합니다.</li>
          </ul>
          <h3>드럼 스텝 그리드 (2x8)</h3>
          <ul>
            <li>선택한 트랙의 16분음표 스텝을 켜고 끕니다. 켜진 셀은 점등되고 굵은 테두리는 현재 재생 위치를 나타냅니다.</li>
            <li>장비가 보내는 96 PPQ MIDI 클럭에 맞춰 움직이며, 본체 PLAY 버튼으로 시작해도 함께 따라갑니다.</li>
            <li><em>스텝은 GUI 레이어에서 재생됩니다.</em> 장비 내부 패턴 메모리에 직접 기록하려면 본체 REC를 켜고 CLI/MCP의 <b>st rec-steps</b>를 실행하세요.</li>
          </ul>
          <h3>REC + C4 ~ B4</h3>
          <ul>
            <li><b>REC</b>(주황)를 켜고 재생 중에 스케일 음표를 누르면 현재 스텝에 기록됩니다.</li>
            <li>장비 본체의 REC도 함께 켜져 있으면 동일한 Note On이 장비 패턴 메모리에도 동시에 기록됩니다.</li>
            <li>REC를 끄고 누르면 녹음 없이 청음만 합니다.</li>
          </ul>
          <h3>SOUND DESIGN &amp; FX</h3>
          <ul>
            <li><b>SD 노브 4개</b> 선택 트랙의 CUTOFF / RESO / ATTACK / DECAY를 조절합니다(CC 74/71/73/75).</li>
            <li><b>FX 노브</b> 이펙트 종류(카테고리 x 프리셋)를 SysEx로 변경합니다.</li>
            <li><b>MST / SINGLE</b> 이펙트 적용 범위를 전환하는 고정식 슬라이드 스위치입니다.</li>
            <li><b>터치 슬라이더</b> PAGE 1: FX LEVEL, 그리고 누르는 동안만 작동하고 놓으면 0으로 돌아가는 HIGH PASS 및 RPT. PAGE 2: 할당 파라미터 1/2/3.</li>
          </ul>
        {:else}
          <p class="flow">믹서 뷰는 11개 트랙 스트립, 드럼 패드, 아르페지오 블록, 트랜스포트, 프로젝트 로더를 한 화면에 모아 실시간 연주와 믹싱에 집중합니다.</p>
          <h3>트랙 스트립</h3>
          <ul>
            <li><b>volume / pan</b> 슬라이더로 각 트랙의 CC 7 / CC 10을 조절합니다.</li>
            <li><b>M</b> 뮤트, <b>S</b> 솔로, <b>HIT</b> 루트 노트 청음.</li>
          </ul>
          <h3>DRUMS 구역</h3>
          <ul>
            <li><b>패드 7개</b> 드럼 트랙(채널 1-7)을 즉시 발음합니다.</li>
          </ul>
          <h3>SYNTH / SAMPLER 구역</h3>
          <ul>
            <li><b>ARP 블록</b> 선택한 신스 트랙(synth1/synth2/dx)의 아르페지오 템플릿, 게이트, 스피드, 모노/폴리/코드 모드, 포르타멘토를 조절합니다.</li>
          </ul>
          <h3>MASTER 구역</h3>
          <ul>
            <li><b>ALL 노브</b> 11개 전체 트랙에 볼륨(CC 7)을 브로드캐스트합니다.</li>
            <li><b>TRANSPORT</b> 시퀀서 재생 및 정지.</li>
            <li><b>PROJECT P1-P8</b> 프로젝트 1-8 로드(전환 시 현재 프로젝트 버퍼 자동 저장).</li>
            <li><b>IN 로그</b> 장비가 보내는 최근 MIDI 이벤트를 표시합니다.</li>
          </ul>
          <h3>장비 패턴 메모리에 작곡하기</h3>
          <ul>
            <li>장비 내부 패턴 메모리에 스텝을 새기려면 본체 패널에서 REC를 켜고 CLI/MCP의 <b>st rec-steps</b>를 실행하세요(연속 테이크는 <code>--keep-playing</code> 사용).</li>
          </ul>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 14, 12, 0.6);
    z-index: 30;
  }
  .panel {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(460px, 94vw);
    background: var(--chassis-deep);
    color: #d8d5ce;
    border-left: 2px solid #000;
    overflow-y: auto;
    padding: 0 0 24px;
  }
  header {
    position: sticky;
    top: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--chassis-deep);
    padding: 14px 18px 10px;
    border-bottom: 1px solid #3a382f;
  }
  h2 {
    margin: 0;
    font-size: 14px;
    letter-spacing: 0.18em;
    color: var(--orange);
  }
  .actions {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .lbtn,
  .x {
    background: transparent;
    border: 1px solid #4a473f;
    color: #d8d5ce;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    cursor: pointer;
  }
  .lbtn.on {
    background: var(--orange);
    border-color: var(--orange-deep);
    color: #fff;
  }
  .body {
    padding: 6px 18px 0;
    line-height: 1.55;
    font-size: 13px;
  }
  .flow {
    color: var(--led-cyan);
    margin: 12px 0 4px;
  }
  h3 {
    margin: 18px 0 4px;
    font-size: 11px;
    letter-spacing: 0.2em;
    color: #97937f;
  }
  ul {
    margin: 0;
    padding-left: 16px;
  }
  li {
    margin: 4px 0;
  }
  b {
    color: #fff;
  }
  em {
    color: var(--led-yellow);
    font-style: normal;
  }
  code {
    background: #211f1b;
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 12px;
  }
</style>
