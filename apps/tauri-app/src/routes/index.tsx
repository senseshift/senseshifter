import { createFileRoute } from '@tanstack/react-router'
import { useState } from "react";
import reactLogo from "../assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@senseshifter/ui/components/card";
import { Button } from "@senseshifter/ui/components/button";
import { Input } from "@senseshifter/ui/components/input";

export const Route = createFileRoute('/')({
  component: Index,
})

function Index() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-background to-muted p-6">
      <div className="container mx-auto max-w-4xl">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-primary to-primary/60 bg-clip-text text-transparent mb-4">
            Welcome to Tauri + React
          </h1>
          <p className="text-muted-foreground text-lg">
            Built with modern tools and beautiful components
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <Card className="group hover:shadow-lg transition-shadow">
            <CardHeader className="text-center pb-3">
              <div className="mx-auto w-16 h-16 mb-3 flex items-center justify-center bg-orange-100 rounded-full group-hover:scale-110 transition-transform">
                <img src="/vite.svg" className="w-10 h-10" alt="Vite logo" />
              </div>
              <CardTitle className="text-orange-600">Vite</CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription className="text-center">
                Lightning fast build tool for modern web development
              </CardDescription>
              <div className="mt-4 text-center">
                <Button variant="outline" size="sm" asChild>
                  <a href="https://vitejs.dev" target="_blank" rel="noopener noreferrer">
                    Learn More
                  </a>
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card className="group hover:shadow-lg transition-shadow">
            <CardHeader className="text-center pb-3">
              <div className="mx-auto w-16 h-16 mb-3 flex items-center justify-center bg-blue-100 rounded-full group-hover:scale-110 transition-transform">
                <img src="/tauri.svg" className="w-10 h-10" alt="Tauri logo" />
              </div>
              <CardTitle className="text-blue-600">Tauri</CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription className="text-center">
                Build secure, cross-platform desktop apps with web technologies
              </CardDescription>
              <div className="mt-4 text-center">
                <Button variant="outline" size="sm" asChild>
                  <a href="https://tauri.app" target="_blank" rel="noopener noreferrer">
                    Learn More
                  </a>
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card className="group hover:shadow-lg transition-shadow">
            <CardHeader className="text-center pb-3">
              <div className="mx-auto w-16 h-16 mb-3 flex items-center justify-center bg-cyan-100 rounded-full group-hover:scale-110 transition-transform">
                <img src={reactLogo} className="w-10 h-10 animate-spin" alt="React logo" />
              </div>
              <CardTitle className="text-cyan-600">React</CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription className="text-center">
                A JavaScript library for building user interfaces
              </CardDescription>
              <div className="mt-4 text-center">
                <Button variant="outline" size="sm" asChild>
                  <a href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
                    Learn More
                  </a>
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>

        <Card className="max-w-md mx-auto">
          <CardHeader>
            <CardTitle>Greeting Demo</CardTitle>
            <CardDescription>
              Enter your name below to get a personalized greeting from Tauri
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form
              className="space-y-4"
              onSubmit={(e) => {
                e.preventDefault();
                greet();
              }}
            >
              <div className="space-y-2">
                <label htmlFor="greet-input" className="text-sm font-medium">
                  Your Name
                </label>
                <Input
                  id="greet-input"
                  value={name}
                  onChange={(e) => setName(e.currentTarget.value)}
                  placeholder="Enter your name..."
                />
              </div>
              <Button type="submit" className="w-full">
                Greet Me
              </Button>
            </form>
            {greetMsg && (
              <div className="mt-4 p-4 bg-primary/10 rounded-md border border-primary/20">
                <p className="text-primary font-medium">{greetMsg}</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}