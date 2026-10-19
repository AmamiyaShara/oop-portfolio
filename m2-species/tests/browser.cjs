const path=require("node:path"),assert=require("node:assert/strict");
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||path.join(process.env.USERPROFILE,".cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright"));
const output=process.env.QA_OUTPUT||require("node:os").tmpdir();
(async()=>{
  const browser=await chromium.launch({executablePath:"C:/Program Files/Google/Chrome/Application/chrome.exe",headless:true});
  try{
    const page=await browser.newPage({viewport:{width:1440,height:900}});
    const errors=[];page.on("pageerror",e=>errors.push(e.message));
    await page.goto("http://127.0.0.1:4173/m2-species/");
    await page.waitForFunction(()=>window.__windSacState?.rust);
    const shot=async name=>page.screenshot({path:path.join(output,"rust-"+name+".png")});
    await shot("overview");
    assert.equal(await page.evaluate(()=>__rustEngine.check_model()),7);
    await page.selectOption("#preview-stage","10");
    await page.selectOption("#preview-cycle","7");
    await page.selectOption("#preview-day","7");
    await page.waitForTimeout(150);
    const frozen=await page.evaluate(()=>__windSacState.time);
    await shot("flower");
    await page.waitForTimeout(100);
    assert.equal(await page.evaluate(()=>__windSacState.time),frozen);
    for(let i=0;i<16;i++){
      await page.selectOption("#preview-stage",String(i));
      await page.waitForTimeout(50);
      const s=await page.evaluate(()=>__windSacState);
      assert.ok(s.eyeY<=-18,"Camera underground");
      if(i>=5&&i<=7)assert.equal(s.observed.growth,0);
    }
    await page.selectOption("#preview-stage","12");
    await page.waitForTimeout(100);await shot("reproduction");
    await page.mouse.move(900,450);await page.mouse.down({button:"right"});
    await page.mouse.move(1200,850,{steps:12});await page.mouse.up({button:"right"});
    assert.ok(await page.evaluate(()=>__windSacState.eyeY<=-18 && __windSacState.pitch<=-0.1));
    await page.selectOption("#preview-stage","1");
    await page.waitForTimeout(100);await shot("crack");
    await page.click("#resume-button");
    await page.waitForTimeout(100);assert.equal(await page.evaluate(()=>__windSacState.preview),false);
    await page.click("#reset-button");
    await page.evaluate(()=>{
      __rustEngine.test_step(40,1,0.8);
      for(let n=0;n<3;n++){__rustEngine.test_step(9,0,0.8);__rustEngine.test_step(4,1,0.8);}
      for(let d=0;d<6;d++){__rustEngine.test_step(8,1,0.8);__rustEngine.test_step(8,0,0.8);__rustEngine.test_step(0.1,1,0.8);}
    });
    await page.waitForTimeout(100);
    assert.equal(await page.evaluate(()=>__windSacState.seeds.length),1,"Early reproduction");
    await page.evaluate(()=>{__rustEngine.test_step(8,1,0.8);__rustEngine.test_step(8,0,0.8);__rustEngine.test_step(1,1,0.8);});
    await page.waitForTimeout(100);
    assert.equal(await page.evaluate(()=>__windSacState.seeds.length),3,"No reproduction after day seven");
    await page.evaluate(()=>{
      __rustEngine.test_step(40,1,0.8);
      for(let n=0;n<3;n++){__rustEngine.test_step(9,0,0.8);__rustEngine.test_step(4,1,0.8);}
      for(let d=0;d<7;d++){__rustEngine.test_step(8,1,0.8);__rustEngine.test_step(8,0,0.8);__rustEngine.test_step(0.2,1,0.8);}
    });
    await page.waitForTimeout(100);
    assert.ok(await page.evaluate(()=>__windSacState.seeds.some(s=>s.generation===1&&s.offspring>0)),"Descendants cannot reproduce");
    await shot("colony");
    await page.click("#reset-button");await page.check("#view-individual");
    const colors=await page.evaluate(()=>{
      noLoop();draw();const gl=drawingContext,p=new Uint8Array(gl.drawingBufferWidth*gl.drawingBufferHeight*4);
      gl.readPixels(0,0,gl.drawingBufferWidth,gl.drawingBufferHeight,gl.RGBA,gl.UNSIGNED_BYTE,p);
      const colors=new Set();for(let i=0;i<p.length;i+=160)colors.add(p[i]+","+p[i+1]+","+p[i+2]);loop();return colors.size;
    });
    assert.ok(colors>20,"Blank canvas");
    await page.setViewportSize({width:390,height:844});
    await page.selectOption("#preview-stage","10");await page.waitForTimeout(150);await shot("mobile");
    const overflow=await page.evaluate(()=>[...document.querySelectorAll(".controls input,.controls button,.phase-browser select,#resume-button,.specimen")].filter(el=>{
      const r=el.getBoundingClientRect();return r.right>innerWidth+1||r.left<0||r.bottom>innerHeight;
    }).map(el=>el.id||el.className));
    assert.deepEqual(overflow,[]);assert.deepEqual(errors,[]);
    console.log("PASS: Rust WASM, previews, pause/resume, reproduction, descendants, camera, WebGL pixels, mobile");
  }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
