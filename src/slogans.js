// Slogans shown at random under the title, always in English whatever the interface language.
export const slogans = [
  'Praise the Squat!',
  'You Died, Do 20',
  'Prepare to Sweat Edition',
  'Don\'t you dare go hollow. One set is already a win.',
  'Thou hast died, so thou shalt squat.',
  'Ashen one, hast thou stretched?',
  'Your soul level went up, and so did your heart rate.',
  'Victory achieved. Penalty still due.',
  'Estus heals HP, not posture.',
  'Your i-frames won\'t save you from the next set.',
  'Resting at a bonfire respawns the enemies, and the reps.',
  'No summon for pull-ups, so every rep is your own victory.',
  'Lose your souls, keep your gains.',
  'Your bloodstain can wait until the set is done.',
  'Wait for the green bar to refill before the next set.',
  'A cheesed boss still drops its Great Soul, and an easy set still builds muscle.',
  'Even Otzdarva got hit while learning.',
  'Visions of leg day…',
  'Humanity restored. Hydration recommended.',
  'The Fire Keeper cannot level your legs.',
  'Firelink Shrine is for resting, and so is the time between sets.',
  'Try jumping. Then try jumping jacks.',
  'Illusory wall ahead: lean on it for a wall sit.',
  'Mimics lie about being chests, but a squat has never lied about being a squat.',
  'Endurance is a stat, level it up one set at a time.',
  'No Red Tearstone Ring here: stop before you\'re in the red.',
  'Ornstein and Smough: two bosses, two sets.',
  'Jolly cooperation includes the cooldown stretch.',
  'No respec needed: your build grows one set at a time.',
  'Enough poise to tank a greatsword, still wobbling on one leg.',
  'Havel\'s set is heavy: lift with your legs, not with your back.',
  'Siegmeyer naps between fights, so you can rest between sets.',
  'Patches swears this set has only ten reps.',
  'Foul Tarnished, in search of proper form.',
  'Torrent can\'t carry you on leg day.',
  'Hesitation is defeat. Rest is not.',
  'Every death spreads Dragonrot, and a set of squats.',
  'Fill their posture bar, then fix your own.',
  'A hunter must hunt, and a squatter must squat.',
  'Nobody dodges Waterfowl Dance the first time.',
  'Someone beat Malenia with a jar for a helmet, so pajamas count as endgame gear.',
  'Let Me Solo Her got good through practice, not exhaustion.',
  'Git gud at being proud of yourself.',
  'The Mimic Tear is the best spirit ash because it\'s you. Just saying.',
  'Summoning isn\'t cheating, and neither are knee push-ups.',
  'Playing a mage isn\'t cheating, and neither is training with light weights.',
  'Leave messages like a spotter: there to help, not to snicker.',
  'Amazing effort ahead. Therefore, try rest.',
  'Here, there\'s an easy mode.',
];

// Slogans already shown since the list was last cleared, kept from one launch to the next.
const SEEN_KEY = 'squatsouls.seenSlogans';

// Picks at random a slogan not shown yet, and starts over from the whole list once all have been.
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
