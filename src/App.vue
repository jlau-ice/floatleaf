<script setup lang="ts">
import OverlayReader from './components/OverlayReader.vue';
import SettingsWindow from './components/SettingsWindow.vue';
const overlay = new URLSearchParams(location.search).get('window') === 'overlay';
document.documentElement.classList.toggle('overlay-root', overlay);
</script>
<template><OverlayReader v-if="overlay" /><SettingsWindow v-else /></template>
<style>
:root {
  font-family: 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #203b32;
  background: #f4f2e9;
  font-synthesis: none;
  -webkit-font-smoothing: antialiased;
}
* {
  box-sizing: border-box;
}
body {
  margin: 0;
}
.overlay-root,
.overlay-root body,
.overlay-root #app {
  background: transparent !important;
  overflow: hidden;
}
.workspace {
  max-width: 1040px;
  margin: auto;
  padding: 30px 44px;
}
header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: center;
  font-size: 11px;
}
.wordmark {
  letter-spacing: 0.2em;
  font-weight: 650;
}
.edition {
  color: #596a5e;
}
.intro {
  padding: 32px 0 28px;
}
.eyebrow {
  font-size: 12px;
  letter-spacing: 0.16em;
  color: #596a5e;
}
h1 {
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
  font-size: clamp(38px, 6vw, 58px);
  font-weight: 500;
  line-height: 1.2;
  margin: 16px 0;
  letter-spacing: -0.04em;
}
h1 span {
  color: #ab7247;
}
.description {
  font-size: 14px;
  line-height: 2;
  margin: 0;
}
kbd {
  font-family: inherit;
  font-size: 0.9em;
  border: 1px solid #c5cabd;
  border-radius: 4px;
  padding: 2px 5px;
}
.row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  border-top: 1px solid #c8cec0;
  padding: 19px 0;
}
h2 {
  font-size: 15px;
  font-weight: 550;
  margin: 0 0 7px;
}
.row p {
  font-size: 12px;
  line-height: 1.8;
  margin: 0;
  color: #58665b;
}
button {
  border: 1px solid #203b32;
  background: #203b32;
  color: #fffdf4;
  border-radius: 4px;
  padding: 9px 12px;
  font: inherit;
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
}
button:hover:enabled {
  background: #315343;
  color: #fffdf4;
}
button:focus-visible {
  outline: 3px solid #ad774f;
  outline-offset: 3px;
}
button:disabled {
  opacity: 0.45;
  cursor: default;
}
.secondary {
  background: transparent;
  color: #203b32;
  border-color: #a7b4a5;
}
.actions,
.segmented {
  display: flex;
  gap: 8px;
}
.segmented {
  border: 1px solid #a7b4a5;
  border-radius: 5px;
  gap: 0;
  padding: 3px;
}
.segmented button {
  background: transparent;
  color: #203b32;
  border-color: transparent;
}
.segmented button[aria-pressed='true'] {
  background: #203b32;
  color: #fffdf4;
}
footer {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 14px;
  font-size: 11px;
  color: #58665b;
}
.footnote {
  margin-left: auto;
}
.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #839080;
}
.status-dot.visible {
  background: #328452;
}
.notice {
  border-left: 3px solid #ab7247;
  padding: 10px;
  font-size: 12px;
  line-height: 1.6;
  background: #ede6d8;
}
.error {
  color: #873b25;
}
@media (max-width: 700px) {
  .workspace {
    padding: 24px;
  }
  .row {
    align-items: flex-start;
    flex-wrap: wrap;
  }
  .footnote {
    margin-left: 0;
    width: 100%;
  }
}
</style>
