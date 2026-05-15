import { ref } from 'vue';

const showUpdateDialog = ref(false);
const isChecking = ref(false);
const updateInfo = ref({
  version: '',
  localVersion: '',
  date: '',
  notes: [],
  url: '',
  changelog: []
});

export function useUpdateStore() {
  return {
    showUpdateDialog,
    isChecking,
    updateInfo
  };
}
