import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { uploadImage, imageUrl } from '@/lib/deviceApi'
import { makeElement, imagePlacement } from '@/stores/elementFactory'

// Resolve an image URL to its natural pixel size by loading it off-screen.
export function naturalSize(src: string): Promise<{ w: number; h: number }> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve({ w: img.naturalWidth, h: img.naturalHeight })
    img.onerror = () => reject(new Error('image failed to load'))
    img.src = src
  })
}

// Upload a file and drop it onto the current page as a centred, aspect-correct
// image element, then leave the select tool active so it's ready to move/resize.
// Returns the new element id, or null if the upload failed (device offline).
export async function addImageFromFile(file: File): Promise<string | null> {
  const store = usePagesStore()
  const opts = useToolOptionsStore()

  const id = await uploadImage(file)
  if (!id) return null

  // Fall back to a square if the device can't serve the image back to us.
  const natural = await naturalSize(imageUrl(id)).catch(() => ({ w: 1, h: 1 }))
  const rect = imagePlacement(natural.w, natural.h, store.pageSize)

  const el = makeElement(
    'image',
    {
      calendarVariant: opts.calendarVariant,
      colour: opts.colour,
      feedId: opts.feedId,
      font: opts.font,
      align: opts.align,
      imageId: id,
    },
    store.pageSize,
    rect,
  )
  store.addElement(el)
  store.setActiveTool('select')
  return el.id
}
