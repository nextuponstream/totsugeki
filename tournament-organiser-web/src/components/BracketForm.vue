<template>
  <div class="grid grid-cols-1 gap-2">
    <div class="text-2xl">
      {{ t("bracketForm.title") }}
    </div>
    <div class="flex gap-2">
      <div class="max-w-2">
        <label>{{ t("bracketForm.nameLabel") }}</label>
        <FormInput
          v-model="bracketName"
          :placeholder="t('bracketForm.namePlaceholder')"
          @keydown.enter="submitNewBracket"
        />
      </div>
      <SubmitBtn
        class="self-end"
        :disabled="bracketName.length === 0"
        @click="submitNewBracket"
      />
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n({});

const bracketName = ref("");

const emit = defineEmits(["newBracket"]);

function submitNewBracket() {
  if (bracketName.value.length > 0) {
    emit("newBracket", bracketName.value);
  }
}
</script>
