import { createFileRoute } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { Radio, Server, AlertCircle, CheckCircle2, Clock, XCircle } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { PageHeader } from '../../components'
import { Button } from '@senseshifter/ui/components/button'
import { Card } from '@senseshifter/ui/components/card'
import { Switch } from '@senseshifter/ui/components/switch'
import { Separator } from '@senseshifter/ui/components/separator'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@senseshifter/ui/components/collapsible'

export const Route = createFileRoute('/settings/osc')({
  component: SettingsOSC,
})

interface AppConfig {
  oscEnabled: boolean
  oscServers: Record<string, OscServerRuntimeConfig>
}

interface OscServerRuntimeConfig {
  id: string
  name: string
  enabled: boolean
  config: {
    server: {
      udp: string[]
      tcp: string[]
    }
    routes: Array<{
      address: string
      stopPropagation: boolean
      forward: Array<{
        target: {
          type: string
          to: string
        }
        rewrite?: string | null
      }>
    }>
    connectionManager: {
      packetBufferSize: number
      maxConcurrentReconnections: number
    }
  }
}

interface ServerStatusEvent {
  serverId: string
  serverName: string
  enabled: boolean
  running: boolean
}

interface ConnectionStatusEvent {
  serverId: string
  targetName: string
  address: string
  transport: string
  status: 'Online' | 'Offline' | 'Reconnecting' | 'Failed'
  nextAttemptAt?: number
}

function SettingsOSC() {
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [serverStatuses, setServerStatuses] = useState<ServerStatusEvent[]>([])
  const [connectionStatuses, setConnectionStatuses] = useState<Record<string, ConnectionStatusEvent[]>>({})
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Load initial config and server statuses
  useEffect(() => {
    // Load them separately to see which one fails
    const loadConfig = async () => {
      const timeout = (ms: number) => new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Timeout')), ms)
      )

      try {
        console.log('Calling get_osc_config...')
        const oscConfig = await Promise.race([
          invoke<AppConfig>('get_osc_config'),
          timeout(5000)
        ]) as AppConfig
        console.log('Loaded config:', oscConfig)
        setConfig(oscConfig)
      } catch (err) {
        console.error('Failed to load config:', err)
        setError(`Config error: ${err}`)
        setIsLoading(false)
        return
      }

      try {
        console.log('Calling get_server_statuses...')
        const statuses = await Promise.race([
          invoke<ServerStatusEvent[]>('get_server_statuses'),
          timeout(5000)
        ]) as ServerStatusEvent[]
        console.log('Loaded statuses:', statuses)
        setServerStatuses(statuses)
        setIsLoading(false)
      } catch (err) {
        console.error('Failed to load statuses:', err)
        setError(`Status error: ${err}`)
        setIsLoading(false)
      }
    }

    // Add a small delay before starting to ensure the backend is ready
    const timer = setTimeout(loadConfig, 100)
    return () => clearTimeout(timer)
  }, [])

  // Listen for events from backend
  useEffect(() => {
    const unsubscribe = listen('osc-event', (event) => {
      console.log('Received OSC event:', event)
      const data = event.payload as ServerStatusEvent | ConnectionStatusEvent

      if ('running' in data) {
        // Server status event
        const statusEvent = data as ServerStatusEvent
        console.log('Server status update:', statusEvent)
        setServerStatuses(prev => 
          prev.map(s => s.serverId === statusEvent.serverId ? statusEvent : s)
        )
      } else if ('targetName' in data) {
        // Connection status event
        const connEvent = data as ConnectionStatusEvent
        console.log('Connection status update:', connEvent)
        console.log('Current connection statuses before update:', connectionStatuses)
        setConnectionStatuses(prev => {
          const updated = {
            ...prev,
            [connEvent.serverId]: [
              ...(prev[connEvent.serverId] || []).filter(c => c.targetName !== connEvent.targetName),
              connEvent
            ]
          }
          console.log('Updated connection statuses:', updated)
          return updated
        })
      } else {
        console.warn('Unknown event type:', data)
      }
    })

    return () => {
      unsubscribe.then(fn => fn())
    }
  }, [])

  const handleGlobalToggle = async (enabled: boolean) => {
    try {
      await invoke('toggle_global_osc', { enabled })
      if (config) {
        setConfig({ ...config, oscEnabled: enabled })
      }
    } catch (err) {
      setError(err as string)
    }
  }

  const handleServerToggle = async (serverId: string, enabled: boolean) => {
    try {
      await invoke('toggle_osc_server', { serverId, enabled })
      // Update local config
      if (config && config.oscServers[serverId]) {
        setConfig({
          ...config,
          oscServers: {
            ...config.oscServers,
            [serverId]: {
              ...config.oscServers[serverId],
              enabled
            }
          }
        })
      }
    } catch (err) {
      setError(err as string)
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case 'online':
        return <CheckCircle2 className="h-4 w-4 text-green-500" />
      case 'offline':
        return <XCircle className="h-4 w-4 text-red-500" />
      case 'reconnecting':
        return <Clock className="h-4 w-4 text-yellow-500" />
      case 'failed':
        return <AlertCircle className="h-4 w-4 text-red-500" />
      default:
        console.warn('Unknown connection status:', status)
        return <XCircle className="h-4 w-4 text-gray-400" />
    }
  }

  if (isLoading) {
    return (
      <div className="flex flex-1 flex-col gap-4 p-4">
        <PageHeader 
          title="OSC Settings"
          description="Configure Open Sound Control settings"
        />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <Radio className="h-8 w-8 animate-pulse mx-auto mb-2 text-muted-foreground" />
            <p className="text-muted-foreground">Loading OSC configuration...</p>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex flex-1 flex-col gap-4 p-4">
        <PageHeader 
          title="OSC Settings"
          description="Configure Open Sound Control settings"
        />
        <Card className="p-6">
          <div className="flex items-center gap-2 text-red-600">
            <AlertCircle className="h-5 w-5" />
            <h3 className="font-semibold">Error</h3>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">{error}</p>
          <Button 
            className="mt-4" 
            onClick={() => window.location.reload()}
            variant="outline"
          >
            Retry
          </Button>
        </Card>
      </div>
    )
  }

  if (!config || typeof config.oscServers === 'undefined') {
    return (
      <div className="flex flex-1 flex-col gap-4 p-4">
        <PageHeader 
          title="OSC Settings"
          description="Configure Open Sound Control settings"
        />
        <Card className="p-6">
          <div className="flex items-center gap-2 text-yellow-600">
            <AlertCircle className="h-5 w-5" />
            <h3 className="font-semibold">Invalid Configuration</h3>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            The OSC configuration is not properly initialized.
          </p>
          <Button 
            className="mt-4" 
            onClick={() => window.location.reload()}
            variant="outline"
          >
            Reload
          </Button>
        </Card>
      </div>
    )
  }

  return (
    <div className="flex flex-1 flex-col gap-4 p-4">
      <PageHeader 
        title="OSC Settings"
        description="Configure Open Sound Control settings"
      />

      {/* OSC Server Collapsible */}
      <Collapsible defaultOpen>
        <Card className="p-0">
          <CollapsibleTrigger className="w-full px-6 py-3 hover:bg-muted/50 transition-colors">
            <div className="flex items-center justify-between w-full">
              <div className="flex items-center gap-3">
                <Server className="h-5 w-5 text-muted-foreground" />
                <div className="text-left">
                  <h3 className="font-semibold">OSC Server</h3>
                  <p className="text-sm text-muted-foreground">
                    Enable or disable OSC server functionality
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Switch
                  checked={config.oscEnabled}
                  onCheckedChange={handleGlobalToggle}
                  onClick={(e) => e.stopPropagation()}
                />
              </div>
            </div>
          </CollapsibleTrigger>
          <CollapsibleContent className="px-6 pb-3">
              <div className="space-y-3">
                <Separator />
                <div>
                  <h4 className="font-medium mb-3">Server Instances</h4>
                  <div className="space-y-3">

                    {Object.entries(config.oscServers || {}).map(([serverId, serverConfig]) => {
                      const status = serverStatuses.find(s => s.serverId === serverId)
                      const connections = connectionStatuses[serverId] || []
                      console.log(`Rendering server ${serverId}:`, { status, connections })

                      return (
                        <div key={serverId} className="p-4 bg-muted/30 rounded-lg border border-muted">
                          <div className="space-y-3">
                            {/* Server Instance Header */}
                            <div className="flex items-center justify-between">
                              <div className="flex items-center gap-3">
                                <div className="flex items-center gap-2">
                                  <div className={`h-2 w-2 rounded-full ${
                                    status?.running ? 'bg-green-500' : 'bg-gray-400'
                                  }`} />
                                  <h5 className="font-medium">{serverConfig.name}</h5>
                                </div>
                                <span className="text-xs px-2 py-1 bg-background rounded-md border">
                                  {status?.running ? 'Running' : 'Stopped'}
                                </span>
                              </div>
                              <Switch
                                checked={serverConfig.enabled}
                                onCheckedChange={(enabled) => handleServerToggle(serverId, enabled)}
                                disabled={!config.oscEnabled}
                              />
                            </div>

                            {/* Server Configuration Info */}
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                              <div>
                                <h6 className="font-medium mb-1 text-xs text-muted-foreground uppercase tracking-wide">Listening On</h6>
                                <div className="space-y-1">
                                  {(serverConfig.config.server?.udp || []).map((addr, idx) => (
                                    <div key={idx} className="flex items-center gap-2 text-muted-foreground">
                                      <span className="text-xs bg-blue-100 text-blue-700 px-1.5 py-0.5 rounded">UDP</span>
                                      <code className="text-xs">{addr}</code>
                                    </div>
                                  ))}
                                  {(serverConfig.config.server?.tcp || []).map((addr, idx) => (
                                    <div key={idx} className="flex items-center gap-2 text-muted-foreground">
                                      <span className="text-xs bg-purple-100 text-purple-700 px-1.5 py-0.5 rounded">TCP</span>
                                      <code className="text-xs">{addr}</code>
                                    </div>
                                  ))}
                                </div>
                              </div>

                              <div>
                                <h6 className="font-medium mb-1 text-xs text-muted-foreground uppercase tracking-wide">Routes ({(serverConfig.config.routes || []).length})</h6>
                                <div className="space-y-1">
                                  {(serverConfig.config.routes || []).slice(0, 2).map((route, idx) => (
                                    <div key={idx} className="text-muted-foreground">
                                      <code className="text-xs">{route.address || 'N/A'}</code>
                                    </div>
                                  ))}
                                  {(serverConfig.config.routes || []).length > 2 && (
                                    <p className="text-xs text-muted-foreground">
                                      +{(serverConfig.config.routes || []).length - 2} more...
                                    </p>
                                  )}
                                </div>
                              </div>
                            </div>

                            {/* Connection Status */}
                            {console.log('Checking connection status display:', { connectionsLength: connections.length, connections }) || (serverConfig.enabled && connections.length > 0) && (
                              <>
                                <Separator className="bg-muted" />
                                <div>
                                  <h6 className="font-medium mb-2 text-xs text-muted-foreground uppercase tracking-wide">Connection Status</h6>
                                  <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                                    {connections.map((conn, idx) => {
                                      console.log('Rendering connection:', conn)
                                      return (
                                        <div key={idx} className="flex items-center gap-2 p-2 bg-background rounded-md border">
                                          {getStatusIcon(conn.status)}
                                          <div className="flex-1 min-w-0">
                                            <p className="text-xs font-medium truncate">{conn.targetName}</p>
                                            <p className="text-xs text-muted-foreground truncate">
                                              {conn.transport} {conn.address}
                                            </p>
                                          </div>
                                        </div>
                                      )
                                    })}
                                  </div>
                                </div>
                              </>
                            )}
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </div>

                {Object.keys(config.oscServers || {}).length === 0 && (
                  <div className="p-8 text-center border border-dashed rounded-lg">
                    <Server className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
                    <h3 className="font-semibold mb-2">No OSC Server Instances</h3>
                    <p className="text-muted-foreground mb-4">
                      No OSC server instances have been configured yet.
                    </p>
                    <Button variant="outline">Add Server Instance</Button>
                  </div>
                )}
              </div>
            </CollapsibleContent>
        </Card>
      </Collapsible>

      {/* Placeholder for future OSC Client section */}
      <Card className="p-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Radio className="h-5 w-5 text-muted-foreground" />
            <div>
              <h3 className="font-semibold">OSC Client</h3>
              <p className="text-sm text-muted-foreground">
                OSC client functionality (coming soon)
              </p>
            </div>
          </div>
          <Switch
            checked={false}
            disabled={true}
          />
        </div>
      </Card>
    </div>
  )
}