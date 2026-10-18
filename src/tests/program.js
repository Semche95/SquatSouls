import { flushPromises } from '@vue/test-utils';
import { openConfig, save, editDialog, editDialogOpen, editDialogClosed } from './app.js';

// Program tab: the edited game, its sub-tabs and the edit dialogs of its tiles.

// Game shown in the select of the Program tab.
export const shownGame = (w) => w.find('#cfg-game option:checked').text();
export const gameButton = (w, text) => w.findAll('.cfg-game-bar .field button').find((b) => b.text() === text);
// Import of the Games tab, next to New game.
export const importButton = (w) => w.findAll('.cfg-game-bar button').find((b) => b.text() === 'Import');
// Renames the edited game with Rename then Done: the new name waits for Save.
export async function rename(w, name) {
  await gameButton(w, 'Rename').trigger('click');
  await w.find('input[aria-label="Name of the game"]').setValue(name);
  await w.find('.cfg-game-bar form').trigger('submit');
}

// Exercises sub-tab, with the first exercise's dialog open unless asked otherwise.
export async function openExercises({ dialog = true } = {}) {
  const w = await openConfig();
  await w.findAll('.subtabs button')[1].trigger('click');
  if (dialog) await openTile(w.find('.tile-slot .tile'));
  return w;
}
// Opens the edit dialog of an event or an exercise from its tile.
export async function openTile(tile) {
  await tile.trigger('click');
  await editDialogOpen();
}
export async function closeDialog() {
  await editDialog().find('.edit-foot .primary').trigger('click');
  await editDialogClosed();
}
// Closes the dialog, saves, then opens the same tile again: the dialog hides the save bar.
export async function saveAndReopen(w, tile) {
  await closeDialog();
  await save(w).trigger('click');
  await flushPromises();
  await openTile(tile());
}
