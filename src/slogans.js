// Slogans affichés au hasard sous le titre.
export const slogans = [
  "Praise the Squat!",
  "You Died, Do 20",
  "Prepare to Sweat Edition",
  "Don't you dare go hollow. Do another set.",
  "Thou hast died, so thou shalt squat.",
  "Ashen one, hast thou stretched?",
  "Your soul level went up, and so did your heart rate.",
  "Victory achieved. Penalty still due.",
  "Estus heals HP, not posture.",
  "Your i-frames won't save you from the next set.",
  "Resting at a bonfire respawns the enemies, and the reps.",
  "No summon can do your push-ups for you.",
  "Lose your souls, keep your gains.",
  "Your bloodstain can wait until the set is done.",
  "Wait for the green bar to refill before the next set.",
  "Every cheesed boss comes with a free bonus set.",
  "Otzdarva never got hit, so you never skip a set.",
  "Visions of leg day…",
];

// Tire un slogan au hasard.
export function pickSlogan(list = slogans) {
  return list[Math.floor(Math.random() * list.length)] ?? "";
}
