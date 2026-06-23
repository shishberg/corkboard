<script setup lang="ts">
import TopBar from './components/TopBar.vue'
import PageSidebar from './components/PageSidebar.vue'
import ToolRail from './components/ToolRail.vue'
import EditorCanvas from './components/EditorCanvas.vue'
import Timeline from './components/Timeline.vue'
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <TopBar />
    <div class="flex min-h-0 flex-1">
      <PageSidebar />
      <ToolRail />
      <div class="min-w-0 flex-1">
        <EditorCanvas />
      </div>
    </div>
    <Timeline />
  </div>
</template>
