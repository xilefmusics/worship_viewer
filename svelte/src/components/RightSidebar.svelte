<script>
  export let onApiChange;
  export let onCommunicationChange;
  export let onModeChange;
  export let onKeyChange;
  export let onCapoChange;
  export let onFontScaleChange;
  export let currentKey;
  export let currentCapo;
  export let fontScale;
  export let mode;
  export let wsID;
  export let wsConfig;
  export let visible;
  export let apiUrl;
  export let apiPort;
  export let communicationUrl;
  export let communicationPort;
  export let version;
</script>

<style>
  #main {
    border-left: 4px solid #333333;
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .item {
    display: flex;
    flex-wrap: wrap;
  }
  .inneritem {
    padding: 0.8em;
    flex: 1;
  }
</style>

<div id='main' style={!visible && "display: none"}>
  <div id='view-changer-panel' class='item'>
    <button
      class={`inneritem ${mode === 'musican' ? 'selected-button': ''}`}
      on:click={() => onModeChange('musican')}
    >Musican</button>
    <button
      class={`inneritem ${mode === 'singer' ? 'selected-button': ''}`}
      on:click={() => onModeChange('singer')}
    >Singer</button>
    <button
      class={`inneritem ${mode === 'beamer-control' ? 'selected-button': ''}`}
      on:click={() => onModeChange('beamer-control')}
    >Beamer Control</button>
    <button
      class={`inneritem ${mode === 'beamer' ? 'selected-button': ''}`}
      on:click={() => onModeChange('beamer')}
    >Beamer</button>
  </div>
  <div class='item'>
    <button class='inneritem' on:click={() => onKeyChange('-1')}>-</button>
    <button class='inneritem' on:click={() => onKeyChange('Self')}>{currentKey}</button>
    <button class='inneritem' on:click={() => onKeyChange('+1')}>+</button>
  </div>
  <div class='item'>
    <button class='inneritem' on:click={() => onCapoChange(-1)}>-</button>
    <button class='inneritem' on:click={() => onCapoChange(0)}>{currentCapo}</button>
    <button class='inneritem' on:click={() => onCapoChange(+1)}>+</button>
  </div>
  <div class='item'>
    <button class='inneritem' on:click={() => onFontScaleChange('decrement')}>-</button>
    <button class='inneritem' on:click={() => onFontScaleChange('reset')}>{fontScale}</button>
    <button class='inneritem' on:click={() => onFontScaleChange('increment')}>+</button>
  </div>
  <div class='item'>
    <button
       class='inneritem'
      on:click={ () => {
        onApiChange(document.getElementById('input-api-url').value, document.getElementById('input-api-port').value)
      }}>
      Change Api
    </button>
  </div>
  <div class='item'>
    <input id='input-api-url' type='text' value={apiUrl} class='inneritem'/>
    <input id='input-api-port' type='text' value={apiPort} class='inneritem'/>
  </div>
  <div class='item'>
    <button
       class='inneritem'
      on:click={ () => {
        onCommunicationChange(document.getElementById('input-communication-url').value, document.getElementById('input-communication-port').value)
      }}>
      Change Communication Server
    </button>
  </div>
  <div class='item'>
    <input id='input-communication-url' type='text' value={communicationUrl} class='inneritem'/>
    <input id='input-communication-port' type='text' value={communicationPort} class='inneritem'/>
  </div>
  <div class='item'>
    <p class='inneritem'>ID: {wsID}</p>
    <button
      class={`inneritem ${wsConfig.sendControls ? 'selected-button': ''}`}
      on:click={() => wsConfig.sendControls = !wsConfig.sendControls}
    >Send Controls</button>
    <button
      class={`inneritem ${wsConfig.receiveControls ? 'selected-button': ''}`}
      on:click={() => wsConfig.receiveControls = !wsConfig.receiveControls}
    >Receive Controls</button>
  </div>
  <div class='item'>
    <p class='inneritem' style='text-align: center'>Version: {version}</p>
  </div>
</div>
