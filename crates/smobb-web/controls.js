// S.M.O.B.B. web controls: bridge touch + soft keyboard to the keydown events
// that ratzilla listens for on `document`. No wasm<->JS calls — we only
// synthesize keyboard events the running ratatui app already understands.

// Mirror ratzilla's is_mobile() (UA-based) so CSS can switch the cell
// height to match the mobile get_screen_size() /19 divisor. A narrow
// desktop window is NOT mobile and must keep the 20px cell (Task 1/3).
if (/mobile|tablet/i.test(navigator.userAgent)) {
  document.documentElement.classList.add("is-mobile");
}

function dispatchKey(key) {
  document.dispatchEvent(
    new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true })
  );
}

// --- control bar buttons -------------------------------------------------
for (const btn of document.querySelectorAll("#controls button[data-key]")) {
  btn.addEventListener("click", (e) => {
    e.preventDefault();
    dispatchKey(btn.dataset.key);
  });
}

// "Skip": hold to fast-forward (repeat Space while pressed = hold-Space).
const skip = document.getElementById("b-skip");
let skipTimer = null;
function startSkip(e) {
  e.preventDefault();
  if (skipTimer !== null) return;
  skip.classList.add("active");
  dispatchKey(" ");
  skipTimer = setInterval(() => dispatchKey(" "), 50);
}
function stopSkip() {
  if (skipTimer !== null) {
    clearInterval(skipTimer);
    skipTimer = null;
  }
  skip.classList.remove("active");
}
if (skip) {
  skip.addEventListener("pointerdown", startSkip);
  skip.addEventListener("pointerup", stopSkip);
  skip.addEventListener("pointerleave", stopSkip);
  skip.addEventListener("pointercancel", stopSkip);
}

// --- soft keyboard bridge (touch only) -----------------------------------
// On touch, focusing a hidden input raises the soft keyboard. Its beforeinput
// events become synthetic keydowns; we keep the field empty so it's a pure
// event source. We only focus it on touch so a desktop physical keyboard
// (whose keydowns already reach `document`) is never double-fed.
const kbd = document.getElementById("kbd");
if (kbd) {
  kbd.addEventListener("beforeinput", (e) => {
    if (e.inputType === "insertText" && e.data) {
      for (const ch of e.data) dispatchKey(ch);
    } else if (e.inputType === "deleteContentBackward") {
      dispatchKey("Backspace");
    } else if (e.inputType === "insertLineBreak") {
      dispatchKey("Enter");
    }
    e.preventDefault(); // keep #kbd empty
  });

  document.addEventListener(
    "pointerdown",
    (e) => {
      if (e.pointerType === "touch" && !e.target.closest("#controls")) {
        kbd.focus();
      }
    },
    { passive: true }
  );
}

// --- wheel / trackpad scrolling -----------------------------------------
// Bridge scroll gestures to the arrow/page keys the app already handles.
// Scrolls whichever pane has focus (story vs sidebar, chosen via <-/-> or Tab).
let wheelAccum = 0;
document.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault(); // stop the page itself from scrolling
    wheelAccum += e.deltaY;
    const STEP = 24; // px per line of scroll
    if (Math.abs(e.deltaY) >= 240) {
      dispatchKey(e.deltaY > 0 ? "PageDown" : "PageUp");
      wheelAccum = 0;
      return;
    }
    while (Math.abs(wheelAccum) >= STEP) {
      dispatchKey(wheelAccum > 0 ? "ArrowDown" : "ArrowUp");
      wheelAccum -= Math.sign(wheelAccum) * STEP;
    }
  },
  { passive: false }
);

// --- touch-swipe scrolling ----------------------------------------------
let swipeY = null;
document.addEventListener(
  "pointerdown",
  (e) => {
    if (e.pointerType === "touch" && !e.target.closest("#controls")) {
      swipeY = e.clientY;
    }
  },
  { passive: true }
);
document.addEventListener(
  "pointermove",
  (e) => {
    if (swipeY === null || e.pointerType !== "touch") return;
    const SWIPE = 24; // px per scroll line
    let dy = swipeY - e.clientY; // finger up => scroll down
    while (Math.abs(dy) >= SWIPE) {
      dispatchKey(dy > 0 ? "ArrowDown" : "ArrowUp");
      dy -= Math.sign(dy) * SWIPE;
      swipeY -= Math.sign(swipeY - e.clientY) * SWIPE;
    }
  },
  { passive: true }
);
document.addEventListener("pointerup", () => { swipeY = null; });
document.addEventListener("pointercancel", () => { swipeY = null; });

// --- responsive: start with the sidebar hidden on phones -----------------
if (window.matchMedia("(max-width: 700px)").matches) {
  // ratzilla mounts asynchronously; nudge after it's listening.
  window.addEventListener("TrunkApplicationStarted", () =>
    setTimeout(() => dispatchKey("F1"), 100)
  );
}

// --- PWA service worker ---------------------------------------------------
if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("sw.js").catch(() => {});
  });
}
