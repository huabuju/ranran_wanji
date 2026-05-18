import { ref } from 'vue';

const showUpdateDialog = ref(false);
const isChecking = ref(false);
const updateInfo = ref({
  version: '',
  localVersion: '',
  dateVersion: '',
  localDateVersion: '',
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
