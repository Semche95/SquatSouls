// Slogans affichés au hasard sous le titre.
export const slogans = [
  'Praise the Squat!',
  'You Died, Do 20',
  'Prepare to Sweat Edition',
  'Don\'t you dare go hollow. Do another set.',
  'Thou hast died, so thou shalt squat.',
  'Ashen one, hast thou stretched?',
  'Your soul level went up, and so did your heart rate.',
  'Victory achieved. Penalty still due.',
  'Estus heals HP, not posture.',
  'Your i-frames won\'t save you from the next set.',
  'Resting at a bonfire respawns the enemies, and the reps.',
  'No summon can do your push-ups for you.',
  'Lose your souls, keep your gains.',
  'Your bloodstain can wait until the set is done.',
  'Wait for the green bar to refill before the next set.',
  'Every cheesed boss comes with a free bonus set.',
  'Otzdarva never got hit, so you never skip a set.',
  'Visions of leg day…',
  'Humanity restored. Hydration recommended.',
  'The Fire Keeper cannot level your legs.',
  'Firelink Shrine is for resting, and so is the time between sets.',
  'Try jumping. Then try jumping jacks.',
  'Illusory wall ahead: lean on it for a wall sit.',
  'Not every chest is a Mimic, but every chest needs push-ups.',
  'Endurance is a stat, level it up one set at a time.',
  'No Red Tearstone Ring here: stop before you\'re in the red.',
  'Ornstein and Smough: two bosses, two sets.',
  'Jolly cooperation includes the cooldown stretch.',
  'Respec is impossible in real life. Level Vigor.',
  'Poise works on bosses, and on one-leg stands.',
  'Havel\'s set is heavy: lift with your legs, not with your back.',
  'Siegmeyer naps between fights, so you can rest between sets.',
  'Patches swears this set has only ten reps.',
  'Foul Tarnished, in search of proper form.',
  'Torrent can\'t carry you on leg day.',
  'Hesitation is defeat. Start the set.',
  'Every death spreads Dragonrot, and a set of squats.',
  'Fill their posture bar, then fix your own.',
  'A hunter must hunt, and a squatter must squat.',
  'Waterfowl Dance dodged, now dodge the excuses.',
  'A pot on the head, a katana in each hand, and good form on every rep.',
  'Let Me Solo Her got good through practice, not exhaustion.',
];

// Slogans déjà affichés depuis que la liste a été vidée, gardés d'un lancement à l'autre.
const SEEN_KEY = 'squatsouls.seenSlogans';

// Tire au hasard un slogan pas encore affiché, et repart de la liste entière quand tous l'ont été.
export function pickSlogan(list = slogans, storage = localStorage) {
  const seen = new Set(JSON.parse(storage.getItem(SEEN_KEY) ?? '[]'));
  let fresh = list.filter((s) => !seen.has(s));
  if (!fresh.length) {
    seen.clear();
    fresh = list;
  }
  const slogan = fresh[Math.floor(Math.random() * fresh.length)] ?? '';
  seen.add(slogan);
  storage.setItem(SEEN_KEY, JSON.stringify([...seen].filter((s) => list.includes(s))));
  return slogan;
}
