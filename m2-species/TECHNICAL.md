# 風囊生物 / Rust 版

## 執行

從專案根目錄啟動靜態伺服器後，開啟 `/m2-species/`。
目前本機網址：`http://127.0.0.1:4173/m2-species/`。

`pkg/residual_city.wasm` 已包含編譯結果，觀看作品不需要安裝 Rust。
網頁仍使用 p5.js 1.9.4 的繪圖 API；模型、場景、生命流程、棲地分配與鏡頭計算均由 Rust 產生。
`sketch.js` 只負責讀取控制項、載入 WASM、轉送繪圖命令和顯示狀態，沒有另一份 JavaScript 生命狀態機。

## 生命流程

1. 從屋頂裂縫以種子形態出土，長出第一根鉤鎖。
2. 自然風足夠時飄起，找到有空間的風扇框或風口後勾住。
3. 經過三個完整夜晚，依序長出鉤鎖、翅膀並成熟；此時不發芽、不開花、不枯萎。
4. 成熟後開始計算七個完整日夜。日照足夠時發芽、形成花苞並開花；缺光時枯萎。
5. 滿七天才進入繁殖期；風力足夠時釋放兩個較小的後代。後代使用相同的完整生命流程。
6. 本體完全凋謝、脫鉤、落回裂縫。上述過程算一輪，下一輪體型縮小 10%。
7. 每個個體最多完成七輪；第七輪落回裂縫後結束生命。

後代的初始體型是親代當輪體型的 78%。棲地不設定每處兩顆的上限，而是依設備框架周長與個體成熟後的展翅間距安排落腳點；沒有空間的種子會飄離屋頂。
自動日照每 40 秒完成一次日夜；手動調整時以低於 18% 的黑夜、再升至高於 45% 的白天，辨認一次完整夜晚。

## 階段檢視

「階段檢視」選單可直接查看各階段，並選擇第 1–7 輪及成體第 1–7 天。
預覽會暫停並保留即時模擬，使用獨立展示個體；按「返回模擬」即可繼續原本進度。
「個體」視角會依體型調整近看距離；拖曳可旋轉，鏡頭有地板、女兒牆和設備避讓限制。

## 原始碼

- `rust/src/model.rs`：個體生命狀態、日夜計時、繁殖與棲地空間分配。
- `rust/src/render.rs`：屋頂、設備、生物幾何比例及鏡頭避讓。
- `rust/src/lib.rs`：WASM 入口、繪圖命令緩衝區、獨立預覽及狀態輸出。
- `sketch.js`：瀏覽器及 p5.js 介接。
- `index.html`、`theme.css`：控制介面與排版。

## 重建與驗證

在專案根目錄的 PowerShell 執行：

```powershell
./m2-species/build.ps1
node m2-species/tests/check-wasm.cjs
node m2-species/tests/browser.cjs
```

建置腳本優先使用專案 `.tools/` 內的 Rust，否則使用 PATH 中的 Cargo。
其他電腦需先安裝 Rust，並執行 `rustup target add wasm32-unknown-unknown`。
工具鏈及編譯暫存已加入 Git 忽略清單；應保留 `pkg/residual_city.wasm`。

瀏覽器測試需要本機 4173 伺服器、Chrome 與 Playwright，可用 `PLAYWRIGHT_MODULE` 指定 Playwright 路徑，以 `QA_OUTPUT` 指定截圖位置。
測試直接執行已編譯的 Rust WASM，檢查生命週期、後代繁殖、階段預覽、鏡頭界線及手機版控制項。

Rust WebAssembly 目標參考：[Rust 官方文件](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html)。
