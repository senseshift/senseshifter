import { createFileRoute } from '@tanstack/react-router'
import { Monitor } from 'lucide-react'
import { PageHeader, PlaceholderContent } from '../components'

export const Route = createFileRoute('/')({
  component: Index,
})

function Index() {
  return (
    <div className="flex flex-1 flex-col gap-4 p-4">
      <PageHeader 
        title="Devices"
        description="Manage and configure your connected devices"
      />
      <PlaceholderContent 
        icon={Monitor}
        title="No devices found"
        description="Connect a device to get started"
        className="flex-1"
      />
    </div>
  )
}