<script setup lang="ts">
import { ref, computed } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { usePagesStore } from '@/stores/pages'
import { uploadImage, imageUrl } from '@/lib/deviceApi'
import type { ImageEl } from '@/stores/types'

const opts = useToolOptionsStore()
const store = usePagesStore()

const fileInput = ref<HTMLInputElement | null>(null)
const uploading = ref(false)
const error = ref(false)

// The image element currently selected, if any — uploads also replace its src.
const selectedImage = computed((): ImageEl | null => {
  const id = store.selectedElId
  if (!id) return null
  const el = store.selectedPage?.elements.find((e) => e.id === id)
  return el?.type === 'image' ? (el as ImageEl) : null
})

// What to preview: the selected element's image, else the pending upload.
const currentId = computed(() => selectedImage.value?.src || opts.imageId)

async function onPick(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  error.value = false
  uploading.value = true
  const id = await uploadImage(file)
  uploading.value = false
  if (!id) {
    error.value = true
    return
  }
  opts.imageId = id
  if (selectedImage.value) store.setElementSrc(selectedImage.value.id, id)
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <img
      v-if="currentId"
      :src="imageUrl(currentId)"
      class="h-20 w-full rounded border bg-neutral-50 object-contain"
      alt=""
    />
    <button
      type="button"
      class="rounded border px-2 py-1 text-sm hover:bg-neutral-100"
      @click="fileInput?.click()"
    >
      {{ currentId ? 'Replace image…' : 'Choose image…' }}
    </button>
    <input
      ref="fileInput"
      data-role="image-file"
      type="file"
      accept="image/*"
      class="hidden"
      @change="onPick"
    />
    <p v-if="uploading" class="text-xs text-neutral-500">Uploading…</p>
    <p v-if="error" data-role="image-error" class="text-xs text-red-600">
      Upload failed — is the device online?
    </p>
    <p class="text-xs text-neutral-500">
      Pick an image, then draw on the canvas to place it.
    </p>
  </div>
</template>
