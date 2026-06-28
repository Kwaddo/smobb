// S.M.O.B.B. web controls: bridge touch + soft keyboard to the keydown events
// that ratzilla listens for on `document`. No wasm<->JS calls — we only
// synthesize keyboard events the running ratatui app already understands.

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
