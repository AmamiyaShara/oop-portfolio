/* Browser adapter only: UI events and p5 drawing commands produced by Rust/WASM. */
let engine;
let canvas;
let lastRoster = "";
let orbitPointer = null;
let pointerX = 0, pointerY = 0;
let snapshot;
const $ = id => document.getElementById(id);
const decoder = new TextDecoder();
const phaseLabels = {
  soil: "裂縫休眠", emerging: "種子出土", hook: "長出單鉤", drifting: "乘風飄散",
  attaching: "勾住棲地", juvenile: "成熟期", adult: "成體期",
  reproduce: "繁殖散播", withering: "完全凋謝", falling: "落回裂縫", dead: "生命結束",
};
const habitatLabels = ["01 / 排風機", "02 / 排風機", "03 / 百葉風口", "04 / 通風管"];

async function setup() {
  const stage = $("stage");
  canvas = createCanvas(stage.clientWidth, stage.clientHeight, WEBGL);
  canvas.parent(stage);
  pixelDensity(Math.min(window.devicePixelRatio || 1, 2));
  noLoop();
  canvas.elt.addEventListener("contextmenu", e => e.preventDefault());
  canvas.elt.addEventListener("pointerdown", e => {
    if (e.button !== 0 && e.button !== 2) return;
    orbitPointer = e.pointerId; pointerX = e.clientX; pointerY = e.clientY;
    canvas.elt.setPointerCapture(e.pointerId);
  });
  canvas.elt.addEventListener("pointermove", e => {
    if (e.pointerId !== orbitPointer || !engine) return;
    engine.orbit(e.clientX - pointerX, e.clientY - pointerY);
    pointerX = e.clientX; pointerY = e.clientY;
  });
  for (const event of ["pointerup", "pointercancel"]) canvas.elt.addEventListener(event, () => orbitPointer = null);
  $("wind-slider").addEventListener("input", () => $("auto-toggle").checked = false);
  $("sunlight-slider").addEventListener("input", () => $("auto-sun-toggle").checked = false);
  $("reset-button").addEventListener("click", resetSimulation);
  for (const id of ["preview-cycle", "preview-stage", "preview-day"]) $(id).addEventListener("change", updatePreview);
  $("resume-button").addEventListener("click", () => { $("preview-stage").value = "-1"; updatePreview(); });
  try {
    const response = await fetch("pkg/residual_city.wasm?v=19");
    if (!response.ok) throw Error("WASM " + response.status);
    const bytes = await response.arrayBuffer();
    engine = (await WebAssembly.instantiate(bytes, {})).instance.exports;
    resetSimulation();
    window.__rustEngine = engine;
    loop();
  } catch (error) {
    $("state-label").textContent = "載入失敗，請重新整理";
    console.error(error);
  }
}

function resetSimulation() {
  if (!engine) return;
  engine.reset();
  $("wind-slider").value = 35; $("sunlight-slider").value = 72;
  $("auto-toggle").checked = true; $("auto-sun-toggle").checked = false;
  $("view-overview").checked = true;
  $("preview-cycle").value = "1"; $("preview-stage").value = "-1"; $("preview-day").value = "1";
  lastRoster = "";
  updatePreview();
}

function updatePreview() {
  if (!engine) return;
  const preview = Number($("preview-stage").value) >= 0;
  engine.set_preview(Number($("preview-cycle").value), Number($("preview-stage").value), Number($("preview-day").value));
  $("resume-button").hidden = !preview;
  $("preview-cycle").disabled = !preview;
  $("preview-day").disabled = !preview;
  $("individual-select").disabled = preview;
  for (const id of ["wind-slider", "sunlight-slider", "auto-toggle", "auto-sun-toggle"]) $(id).disabled = preview;
  if (preview) $("view-individual").checked = true;
}

function draw() {
  if (!engine) return;
  engine.frame(Math.min(deltaTime / 1000, 0.05), width, height,
    Number($("wind-slider").value) / 100, Number($("sunlight-slider").value) / 100,
    Number($("auto-toggle").checked), Number($("auto-sun-toggle").checked),
    Number($("individual-select").value || 1), Number($("view-individual").checked));
  executeDrawCommands();
  snapshot = JSON.parse(decoder.decode(new Uint8Array(engine.memory.buffer, engine.state_ptr(), engine.state_len())));
  window.__windSacState = snapshot;
  updateReadout(snapshot);
}

function executeDrawCommands() {
  const commands = new Float32Array(engine.memory.buffer, engine.draw_ptr(), engine.draw_len());
  for (let i = 0; i < commands.length;) {
    const op = commands[i++], length = commands[i++];
    const a = commands.subarray(i, i + length); i += length;
    switch (op) {
      case 0: push(); break;
      case 1: pop(); break;
      case 2: translate(...a); break;
      case 3: rotateX(a[0]); break;
      case 4: rotateY(a[0]); break;
      case 5: rotateZ(a[0]); break;
      case 6: scale(...a); break;
      case 7: ambientMaterial(...a); break;
      case 8: box(...a); break;
      case 9: sphere(...a); break;
      case 10: cylinder(...a); break;
      case 11: torus(...a); break;
      case 12: stroke(...a); break;
      case 13: strokeWeight(a[0]); break;
      case 14: line(...a); break;
      case 15: noStroke(); break;
      case 16: fill(...a); break;
      case 17: beginShape(TRIANGLE_STRIP); break;
      case 18: vertex(...a); break;
      case 19: endShape(); break;
      case 20: rotate(a[0], [a[1], a[2], a[3]]); break;
      case 30: background(...a); break;
      case 31: perspective(...a); break;
      case 32: camera(...a); break;
      case 33: ambientLight(...a); break;
      case 34: directionalLight(...a); break;
      default: throw Error("Unknown Rust drawing command: " + op);
    }
  }
}

function updateReadout(state) {
  const roster = state.seeds.map(s => s.id + ":" + s.phase).join(",");
  if (roster !== lastRoster) {
    const select = $("individual-select"), current = select.value;
    select.replaceChildren(...state.seeds.map(s => {
      const option = document.createElement("option"); option.value = s.id;
      option.textContent = "個體 " + String(s.id).padStart(2, "0") + " · 第 " + (s.generation + 1) + " 代";
      if (s.phase === "dead") option.textContent += " · 結束";
      return option;
    }));
    if (state.seeds.some(s => String(s.id) === current)) select.value = current;
    lastRoster = roster;
  }
  const s = state.observed;
  let label = phaseLabels[s.phase];
  if (s.phase === "juvenile") label += " " + s.nights + "/3 夜";
  if (s.phase === "adult") label += " " + (s.days + 1) + "/7 天";
  $("state-label").textContent = (state.preview ? "預覽 · " : "") + label;
  $("cycle-value").textContent = s.cycle + " / 7";
  $("size-value").textContent = Math.round(s.size * 100) + "%";
  $("life-value").textContent = s.phase === "juvenile" ? s.nights + "/3 夜" : (s.phase === "adult" ? s.days + 1 : s.days) + "/7 天";
  $("population-value").textContent = state.population;
  $("habitat-value").textContent = s.habitat >= 0 ? habitatLabels[s.habitat] : "屋頂裂縫 / 自然風";
  $("wind-value").textContent = Math.round(state.wind * 100);
  $("sunlight-value").textContent = Math.round(state.sunlight * 100);
  if ($("auto-toggle").checked && !state.preview) $("wind-slider").value = Math.round(state.wind * 100);
  if ($("auto-sun-toggle").checked && !state.preview) $("sunlight-slider").value = Math.round(state.sunlight * 100);
}

function windowResized() {
  const stage = $("stage");
  resizeCanvas(stage.clientWidth, stage.clientHeight);
}
