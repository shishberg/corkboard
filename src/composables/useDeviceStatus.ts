import { onMounted, onUnmounted, ref } from 'vue'
import { fetchStatus } from '@/lib/deviceApi'
import type { DashboardStatus } from '@/lib/dashboardTypes'

/** Polls `/api/status` on an interval; `status` stays at its last good value
 * while `unreachable` reflects only the most recent attempt. */
export function useDeviceStatus(intervalMs = 5000) {
  const status = ref<DashboardStatus | null>(null)
  const unreachable = ref(false)
  let timer: ReturnType<typeof setInterval> | undefined

  async function refresh() {
    const s = await fetchStatus()
    unreachable.value = s === null
    if (s !== null) status.value = s
  }

  onMounted(() => {
    refresh()
    timer = setInterval(refresh, intervalMs)
  })
  onUnmounted(() => clearInterval(timer))

  return { status, unreachable }
}
