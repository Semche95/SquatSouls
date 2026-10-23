<script setup>
// Frame of the Program and Settings tabs: banners and save bar, limited to the fields of the tab.
import { computed, onBeforeUnmount } from 'vue';
import { normalized, fieldError, withTabDraft } from '../config/configDraft.js';
import { t } from '../i18n/i18n.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { vReveal } from '../ui/reveal.js';
import DisabledTip from './DisabledTip.vue';

const props = defineProps({ settings: Boolean });
const { config, draft, configError, configMsg, notify, tabDraft, tabDirty } = useConfig();
const { saveNow } = useSession();
onBeforeUnmount(() => notify(''));

const pending = computed(() => tabDraft(props.settings));
const dirty = computed(() => tabDirty(props.settings));
// First badly filled field of the tab, which blocks "Save" (empty if everything is valid).
const draftError = computed(() => (pending.value ? fieldError(normalized(pending.value)) : ''));

// Back to the saved values for this tab's fields only.
function resetTab() {
  draft.value = withTabDraft(config.value, draft.value, !props.settings);
}

async function saveConfig() {
  notify('');
  if (!(await saveNow(pending.value))) return;
  resetTab();
  notify(t('config.saved'));
}

function discardChanges() {
  resetTab();
  notify('');
}
</script>

<template>
  <main class="panel config">
    <p v-if="configError" v-reveal class="banner error">{{ configError }}</p>
    <p v-if="configMsg" v-reveal class="banner ok">{{ configMsg }}</p>

    <slot v-if="draft" />

    <div class="row cfg-actions bottom-bar">
      <DisabledTip :tip="dirty ? draftError : t('config.noChanges')" :error="dirty">
        <button class="primary" :disabled="!dirty || !!draftError" @click="saveConfig">{{ t('config.save') }}</button>
      </DisabledTip>
      <DisabledTip :tip="dirty ? '' : t('config.noChanges')">
        <button class="ghost" :disabled="!dirty" @click="discardChanges">{{ t('config.discard') }}</button>
      </DisabledTip>
      <span v-if="dirty" class="unsaved">{{ t('config.unsaved') }}</span>
    </div>
  </main>
</template>
