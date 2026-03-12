import { ROUTE_HOME, TOKEN_KEY } from "@/lib/constants";
import { addToast, Button, Card, CardBody, CardHeader, Input } from "@heroui/react";
import { useState } from "react";
import { useLocation } from "wouter";

type LoginResponse = { token: string };

export default function Login() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [, setLocation] = useLocation();

  const handleLogin = async () => {
    if (!username || !password) return;
    setLoading(true);
    try {
      const res = await fetch("/login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ username, password }),
      });

      if (res.ok) {
        const data = await res.json() as LoginResponse;
        if (data && typeof data === 'object' && 'token' in data && typeof data.token === 'string') {
          localStorage.setItem(TOKEN_KEY, data.token);
          setLocation(ROUTE_HOME);
          addToast({
              title: "Login Successful",
              color: "success"
          });
        } else {
          throw new Error("Invalid response format");
        }
      } else {
        const text = await res.text();
        addToast({
            title: "Login Failed",
            description: text,
            color: "danger"
        });
      }
    } catch (e) {
      console.error(e);
      const errorMessage = e instanceof Error ? e.message : String(e);
      addToast({
          title: "Login Error",
          description: errorMessage,
          color: "danger"
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center h-screen p-4">
      <Card className="w-full max-w-sm">
        <CardHeader className="flex justify-center pb-0">
          <h1 className="text-2xl font-bold">Login</h1>
        </CardHeader>
        <CardBody className="gap-4">
          <Input
            label="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleLogin()}
          />
          <Input
            label="Password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleLogin()}
          />
          <Button color="primary" isLoading={loading} onPress={handleLogin}>
            Login
          </Button>
        </CardBody>
      </Card>
    </div>
  );
}
