import { createFileRoute } from '@tanstack/react-router'
import { Info } from 'lucide-react'
import { PageHeader, PlaceholderContent } from '../components'

export const Route = createFileRoute('/about')({
  component: About,
})

function About() {
  return (
    <div className="flex flex-1 flex-col gap-4 p-4">
      <PageHeader 
        title="About"
        description="Learn more about SenseShifter"
      />
      <PlaceholderContent 
        icon={Info}
        title="SenseShifter v0.1.0"
        description="A modern haptic control application built with Tauri and React"
        className="flex-1"
      />
    </div>
  )
}