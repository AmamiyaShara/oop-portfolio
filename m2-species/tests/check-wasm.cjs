const fs = require("node:fs");
const path = require("node:path");
const assert = require("node:assert/strict");
(async () => {
  const {instance}=await WebAssembly.instantiate(fs.readFileSync(path.join(__dirname,"../pkg/residual_city.wasm")),{});
  const rust=instance.exports;
  assert.equal(rust.check_model(),7);
  rust.reset();rust.frame(0,1440,900,0.35,0.72,0,0,1,0);
  const state=JSON.parse(new TextDecoder().decode(new Uint8Array(rust.memory.buffer,rust.state_ptr(),rust.state_len())));
  assert.equal(state.seeds[0].phase,"soil");
  assert.equal(state.seeds[0].cycle,1);
  assert.equal(state.seeds[0].habitat,-1);
  console.log("PASS: compiled Rust lifecycle, maturation, 7 days, 7 cycles, frame capacity, camera, initial crack seed");
})().catch(e=>{console.error(e);process.exitCode=1;});
