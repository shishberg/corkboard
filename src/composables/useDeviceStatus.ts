import { onMounted, ref } from 'vue'
import { fetchStatus } from '@/lib/deviceApi'
import type { DashboardStatus } from '@/lib/dashboardTypes'

/** Fetches `/api/status` once on load — no background polling, so an idle
 * dashboard tab generates zero ongoing traffic to the device. Call
 * `refresh()` again (e.g. from a button) to get a fresh snapshot. */
export function useDeviceStatus() {
  const status = ref<DashboardStatus | null>(null)
  const unreachable = ref(false)

  async function refresh() {
    const s = await fetchStatus()
    unreachable.value = s === null
    if (s !== null) status.value = s
  }

  onMounted(refresh)

  return { status, unreachable, refresh }
}
