import { useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { toast } from "sonner";
import { Eye, Key, Loader2, Terminal } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { authApi } from "@/lib/api";
import { post, setAuthToken } from "@/lib/api/web-client";
import { webLog } from "@/lib/webLogger";

interface LoginPageProps {
  onLogin: () => void;
}

export function LoginPage({ onLogin }: LoginPageProps) {
  const { t } = useTranslation();
  const [token, setToken] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isRevealingToken, setIsRevealingToken] = useState(false);

  const handleRevealToken = async () => {
    setIsRevealingToken(true);
    webLog.info("login: reveal token requested");
    try {
      const revealedToken = await authApi.generateWebAdminToken();
      setToken(revealedToken);
      webLog.info("login: token revealed");
      toast.success(
        t("login.tokenRevealed", {
          defaultValue: "Token generated and filled",
        }),
      );
    } catch (error) {
      webLog.error("login: reveal token failed", {
        error: error instanceof Error ? error.message : String(error),
      });
      toast.error(
        t("login.tokenRevealFailed", {
          defaultValue: "Failed to reveal token: {{error}}",
          error: error instanceof Error ? error.message : "Unknown error",
        }),
      );
    } finally {
      setIsRevealingToken(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!token.trim()) {
      toast.error(
        t("login.tokenRequired", {
          defaultValue: "Please enter your admin token",
        }),
      );
      return;
    }

    setIsLoading(true);
    webLog.info("login: verifying token");

    try {
      const { valid } = await post<{ valid: boolean }>("/auth/verify", { token: token.trim() });

      if (!valid) {
        throw new Error("Invalid token");
      }

      setAuthToken(token.trim());
      webLog.info("login: success");
      toast.success(t("login.success", { defaultValue: "Login successful" }));
      onLogin();
    } catch (error) {
      webLog.warn("login: failed", {
        error: error instanceof Error ? error.message : String(error),
      });
      toast.error(
        t("login.error", {
          defaultValue: "Login failed",
          error: error instanceof Error ? error.message : "Unknown error",
        }),
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background to-muted p-4">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="w-full max-w-md"
      >
        <Card className="border-2 shadow-xl">
          <CardHeader className="space-y-1 text-center">
            <div className="flex justify-center mb-4">
              <div className="w-16 h-16 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center shadow-lg">
                <span className="text-2xl font-bold text-white">CC</span>
              </div>
            </div>
            <CardTitle className="text-2xl font-bold">CC Switch</CardTitle>
            <CardDescription>
              {t("login.subtitle", { defaultValue: "Web Management Console" })}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="token">
                  {t("login.token", { defaultValue: "Admin Token" })}
                </Label>
                <div className="relative">
                  <Key className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    id="token"
                    type="password"
                    placeholder={t("login.tokenPlaceholder", {
                      defaultValue: "Paste your admin token here",
                    })}
                    value={token}
                    onChange={(e) => setToken(e.target.value)}
                    className="pl-10"
                    disabled={isLoading || isRevealingToken}
                  />
                </div>
                <Button
                  type="button"
                  variant="outline"
                  className="w-full"
                  onClick={handleRevealToken}
                  disabled={isLoading || isRevealingToken}
                >
                  {isRevealingToken ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      {t("login.revealing", { defaultValue: "Generating..." })}
                    </>
                  ) : (
                    <>
                      <Eye className="mr-2 h-4 w-4" />
                      {t("login.revealToken", { defaultValue: "Reveal Token" })}
                    </>
                  )}
                </Button>
              </div>
              <Button type="submit" className="w-full" disabled={isLoading}>
                {isLoading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {t("login.loggingIn", { defaultValue: "Verifying..." })}
                  </>
                ) : (
                  t("login.submit", { defaultValue: "Sign In" })
                )}
              </Button>
            </form>

            <div className="mt-6 p-4 bg-muted rounded-lg">
              <div className="flex items-start gap-3">
                <Terminal className="h-5 w-5 text-muted-foreground mt-0.5" />
                <div className="text-sm text-muted-foreground">
                  <p className="font-medium text-foreground mb-1">
                    {t("login.cliInstructions", {
                      defaultValue: "Generate a token using CLI:",
                    })}
                  </p>
                  <code className="block bg-background px-2 py-1 rounded text-xs">
                    cc-switch-web generate-token
                  </code>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <p className="text-center text-sm text-muted-foreground mt-6">
          {t("login.tokenHelp", {
            defaultValue:
              "Run the command on your server to generate an admin token",
          })}
        </p>
      </motion.div>
    </div>
  );
}

export default LoginPage;
