import { createFileRoute } from '@tanstack/react-router'
import { Radio } from 'lucide-react'
import { PageHeader, PlaceholderContent } from '../../components'

export const Route = createFileRoute('/settings/osc')({
  component: SettingsOSC,
})

function SettingsOSC() {
  return (
    <div className="flex flex-1 flex-col gap-4 p-4">
      <PageHeader 
        title="OSC Settings"
        description="Configure Open Sound Control settings"
      />
      <PlaceholderContent 
        icon={Radio}
        title="OSC Configuration"
        description="Configure your OSC server and client settings here"
        className="flex-1"
      />
    </div>
  )
}