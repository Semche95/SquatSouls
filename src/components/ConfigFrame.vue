<script setup>
// Frame of the Program and Settings tabs: banners and save bar, limited to the fields of the tab.
import { computed, onBeforeUnmount } from 'vue';
import { normalized, numberError, withTabDraft } from '../configDraft.js';
import { t } from '../i18n.js';
import { useConfig } from '../composables/useConfig.js';
import { useSession } from '../composables/useSession.js';
import { vReveal } from '../reveal.js';
import DisabledTip from './DisabledTip.vue';

const props = defineProps({ settings: Boolean });
const { config, draft, configError, configMsg, notify } = useConfig();
const { saveNow } = useSession();
onBeforeUnmount(() => notify(''));

// Configuration this tab would save; the other tab's pending changes stay in the draft.
const pending = computed(() => draft.value && withTabDraft(config.value, draft.value, props.settings));
const dirty = computed(() => !!pending.value && JSON.stringify(pending.value) !== JSON.stringify(config.value));
// First badly filled field of the tab, which blocks "Save" (empty if everything is valid).
const draftError = computed(() => (pending.value ? numberError(normalized(pending.value)) : ''));

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

    <div class="row cfg-actions">
      <DisabledTip :tip="dirty ? draftError : t('config.noChanges')" :error="dirty">
        <button class="primary" :disabled="!dirty || !!draftError" @click="saveConfig">{{ t('config.save') }}</button>
      </DisabledTip>
      <DisabledTip :tip="dirty ? '' : t('config.noChanges')">
        <button class="ghost" :disabled="!dirty" @click="discardChanges">{{ t('config.discard') }}</button>
      </DisabledTip>
    </div>
  </main>
</template>
