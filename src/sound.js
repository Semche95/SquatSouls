// Timer and recovery sounds, synthesized so that no sound file is bundled.
let context;

// Pure tone: a single partial.
const PURE = [[1, 1]];
// Bell: inharmonic partials (frequency ratio, volume), which give its metallic ring.
const BELL = [[1, 1], [2.76, 0.5], [5.4, 0.25], [8.93, 0.12]];

// Notes one after the other, each fading out, every partial of `timbre` playing at once.
function notes(frequencies, length, { gap = 0.18, timbre = PURE } = {}) {
  const AudioContext = window.AudioContext ?? window.webkitAudioContext;
  if (!AudioContext) return;
  context ??= new AudioContext();
  frequencies.forEach((frequency, i) => {
    const at = context.currentTime + i * gap;
    for (const [ratio, level] of timbre) {
      const note = context.createOscillator();
      const volume = context.createGain();
      note.frequency.value = frequency * ratio;
      volume.gain.setValueAtTime(0.0001, at);
      volume.gain.exponentialRampToValueAtTime(0.3 * level, at + 0.01);
      // Higher partials die out sooner, like a struck metal.
      volume.gain.exponentialRampToValueAtTime(0.0001, at + length / ratio ** 0.5);
      note.connect(volume).connect(context.destination);
      note.start(at);
      note.stop(at + length);
    }
  });
}

// Start of the timer: a single short low note, a "go".
export const startSignal = () => notes([523], 0.25);

// End of the timer: two rising notes.
export const chime = () => notes([880, 1320], 0.5);

// End of the recovery: a bell struck twice, like the start of a boxing round.
export const recoveryEnd = () => notes([740, 740], 1.4, { gap: 0.35, timbre: BELL });
