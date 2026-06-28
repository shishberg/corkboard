<script setup lang="ts">
import TopBar from './components/TopBar.vue'
import PageSidebar from './components/PageSidebar.vue'
import ToolRail from './components/ToolRail.vue'
import ToolOptionsBar from './components/ToolOptionsBar.vue'
import EditorCanvas from './components/EditorCanvas.vue'
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <TopBar />
    <div class="flex min-h-0 flex-1">
      <PageSidebar />
      <ToolRail />
      <div class="flex min-w-0 flex-1 flex-col">
        <ToolOptionsBar />
        <div class="min-h-0 flex-1">
          <EditorCanvas />
        </div>
      </div>
    </div>
  </div>
</template>
