// Timer sounds, synthesized so that no sound file is bundled.
let context;

// Called on a user action (starting the timer): a webview may refuse to play sound outside one.
export function unlockSound() {
  const AudioContext = window.AudioContext ?? window.webkitAudioContext;
  if (!AudioContext) return;
  context ??= new AudioContext();
  context.resume();
}

// Notes one after the other, each fading out.
function notes(frequencies, length) {
  if (!context) return;
  frequencies.forEach((frequency, i) => {
    const at = context.currentTime + i * 0.18;
    const note = context.createOscillator();
    const volume = context.createGain();
    note.frequency.value = frequency;
    volume.gain.setValueAtTime(0.0001, at);
    volume.gain.exponentialRampToValueAtTime(0.3, at + 0.02);
    volume.gain.exponentialRampToValueAtTime(0.0001, at + length);
    note.connect(volume).connect(context.destination);
    note.start(at);
    note.stop(at + length);
  });
}

// Start of the timer: a single short low note, a "go".
export const startSignal = () => notes([523], 0.25);

// End of the timer: two rising notes.
export const chime = () => notes([880, 1320], 0.5);
