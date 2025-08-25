import { LucideIcon } from 'lucide-react'
import { cn } from '@senseshifter/ui/lib/utils'

interface PlaceholderContentProps {
  icon?: LucideIcon
  title: string
  description?: string
  children?: React.ReactNode
  className?: string
}

export function PlaceholderContent({ 
  icon: Icon, 
  title, 
  description, 
  children,
  className 
}: PlaceholderContentProps) {
  return (
    <div className={cn("flex items-center justify-center rounded-lg border border-dashed shadow-sm", className)}>
      <div className="flex flex-col items-center gap-4 text-center p-6">
        {Icon && (
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-muted">
            <Icon className="h-6 w-6 text-muted-foreground" />
          </div>
        )}
        <div className="flex flex-col items-center gap-1">
          <h3 className="text-xl font-bold tracking-tight">{title}</h3>
          {description && (
            <p className="text-sm text-muted-foreground max-w-md">
              {description}
            </p>
          )}
        </div>
        {children}
      </div>
    </div>
  )
}