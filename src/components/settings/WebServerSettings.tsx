import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  Globe,
  Play,
  Square,
  ExternalLink,
  Copy,
  Check,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/lib/environment";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export function WebServerSettings() {
  const { t } = useTranslation();
  const [isRunning, setIsRunning] = useState(false);
  const [serverUrl, setServerUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [bindAll, setBindAll] = useState(false);

  useEffect(() => {
    checkServerStatus();
    const interval = setInterval(checkServerStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const checkServerStatus = async () => {
    try {
      const running = await invoke<boolean>("is_web_server_running");
      setIsRunning(running);
      if (running) {
        const url = await invoke<string | null>("get_web_server_url");
        setServerUrl(url);
        const bindAllStatus = await invoke<boolean>("is_web_server_bind_all");
        setBindAll(bindAllStatus);
      } else {
        setServerUrl(null);
      }
    } catch (error) {
      console.error("Failed to check web server status:", error);
    }
  };

  const handleStart = async () => {
    setIsLoading(true);
    try {
      const url = await invoke<string>("start_web_server", { port: null });
      setServerUrl(url);
      setIsRunning(true);
      toast.success(
        t("settings.webServer.started", {
          defaultValue: "Web server started",
        }),
      );
    } catch (error) {
      console.error("Failed to start web server:", error);
      toast.error(
        t("settings.webServer.startFailed", {
          defaultValue: "Failed to start web server",
          error: String(error),
        }),
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleStop = async () => {
    setIsLoading(true);
    try {
      await invoke("stop_web_server");
      setIsRunning(false);
      setServerUrl(null);
      toast.success(
        t("settings.webServer.stopped", {
          defaultValue: "Web server stopped",
        }),
      );
    } catch (error) {
      console.error("Failed to stop web server:", error);
      toast.error(
        t("settings.webServer.stopFailed", {
          defaultValue: "Failed to stop web server",
          error: String(error),
        }),
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleOpenInBrowser = async () => {
    if (serverUrl) {
      try {
        const browserUrl = serverUrl.replace("0.0.0.0", "localhost");
        if (isTauri()) {
          // Use Function constructor to bypass Vite's static analysis
          // This allows the import to be truly dynamic and only resolved at runtime
          const openModule = await new Function(
            'return import("@tauri-apps/plugin-opener")'
          )();
          await openModule.open(browserUrl);
        } else {
          window.open(browserUrl, "_blank");
        }
      } catch (error) {
        console.error("Failed to open browser:", error);
        toast.error(
          t("settings.webServer.openFailed", {
            defaultValue: "Failed to open browser",
          }),
        );
      }
    }
  };

  const handleCopyUrl = async () => {
    if (serverUrl) {
      try {
        await navigator.clipboard.writeText(serverUrl);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
        toast.success(
          t("settings.webServer.urlCopied", {
            defaultValue: "URL copied to clipboard",
          }),
        );
      } catch (error) {
        console.error("Failed to copy URL:", error);
      }
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Globe className="w-5 h-5" />
          {t("settings.webServer.title", {
            defaultValue: "Web Interface",
          })}
        </CardTitle>
        <CardDescription>
          {t("settings.webServer.description", {
            defaultValue:
              "Run a web server alongside the desktop app to access CC Switch from a browser",
          })}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-1">
            <p className="text-sm font-medium">
              {t("settings.webServer.status", { defaultValue: "Status" })}
            </p>
            <p className="text-sm text-muted-foreground">
              {isRunning
                ? t("settings.webServer.running", {
                    defaultValue: "Web server is running",
                  })
                : t("settings.webServer.stopped", {
                    defaultValue: "Web server is stopped",
                  })}
            </p>
          </div>
          <Badge
            variant={isRunning ? "default" : "secondary"}
            className={cn(
              isRunning &&
                "bg-emerald-500 hover:bg-emerald-600 dark:bg-emerald-600 dark:hover:bg-emerald-700",
            )}
          >
            {isRunning
              ? t("settings.webServer.statusRunning", {
                  defaultValue: "Running",
                })
              : t("settings.webServer.statusStopped", {
                  defaultValue: "Stopped",
                })}
          </Badge>
        </div>

        {isRunning && serverUrl && (
          <div className="space-y-2">
            <p className="text-sm font-medium">
              {t("settings.webServer.url", { defaultValue: "Server URL" })}
            </p>
            <div className="flex items-center gap-2">
              <code className="flex-1 px-3 py-2 text-sm bg-muted rounded-md font-mono">
                {serverUrl}
              </code>
              <Button
                variant="outline"
                size="icon"
                onClick={handleCopyUrl}
                title={t("settings.webServer.copyUrl", {
                  defaultValue: "Copy URL",
                })}
              >
                {copied ? (
                  <Check className="w-4 h-4" />
                ) : (
                  <Copy className="w-4 h-4" />
                )}
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={handleOpenInBrowser}
                title={t("settings.webServer.openInBrowser", {
                  defaultValue: "Open in browser",
                })}
              >
                <ExternalLink className="w-4 h-4" />
              </Button>
            </div>
            {bindAll && (
              <p className="text-xs text-muted-foreground flex items-center gap-1">
                <AlertCircle className="w-3 h-3" />
                {t("settings.webServer.bindAllWarning", {
                  defaultValue:
                    "Server is accessible from any device on the network",
                })}
              </p>
            )}
          </div>
        )}

        <div className="flex items-center gap-2">
          {isRunning ? (
            <Button
              variant="destructive"
              onClick={handleStop}
              disabled={isLoading}
              className="w-full"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <Square className="w-4 h-4 mr-2" />
              )}
              {t("settings.webServer.stop", { defaultValue: "Stop Server" })}
            </Button>
          ) : (
            <Button
              onClick={handleStart}
              disabled={isLoading}
              className="w-full"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <Play className="w-4 h-4 mr-2" />
              )}
              {t("settings.webServer.start", { defaultValue: "Start Server" })}
            </Button>
          )}
        </div>

        <div className="p-3 bg-muted rounded-lg">
          <p className="text-xs text-muted-foreground">
            {t("settings.webServer.info", {
              defaultValue:
                "The web interface allows you to access CC Switch from any browser on your network. Use environment variables CC_SWITCH_WEB_PORT and CC_SWITCH_WEB_BIND_ALL to configure the server at startup.",
            })}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
