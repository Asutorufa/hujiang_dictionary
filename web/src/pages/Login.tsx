import { ROUTE_HOME, TOKEN_KEY } from "@/lib/constants";
import { addToast } from "@/components";
import { Button, Card, TextField, Flex, Text, Box } from "@radix-ui/themes";
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
        const data = (await res.json()) as LoginResponse;
        if (
          data &&
          typeof data === "object" &&
          "token" in data &&
          typeof data.token === "string"
        ) {
          localStorage.setItem(TOKEN_KEY, data.token);
          setLocation(ROUTE_HOME);
          addToast({
            title: "Login Successful",
            color: "success",
          });
        } else {
          throw new Error("Invalid response format");
        }
      } else {
        const text = await res.text();
        addToast({
          title: "Login Failed",
          description: text,
          color: "danger",
        });
      }
    } catch (e) {
      console.error(e);
      const errorMessage = e instanceof Error ? e.message : String(e);
      addToast({
        title: "Login Error",
        description: errorMessage,
        color: "danger",
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center h-screen p-4">
      <Card size="4" className="w-full max-w-sm">
        <Flex direction="column" gap="4">
          <Box className="flex justify-center pb-0">
            <Text size="6" weight="bold">
              Login
            </Text>
          </Box>
          <Flex direction="column" gap="3">
            <Flex direction="column" gap="1">
              <Text as="label" size="2" weight="bold">
                Username
              </Text>
              <TextField.Root
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleLogin()}
                size="3"
                variant="surface"
              />
            </Flex>
            <Flex direction="column" gap="1">
              <Text as="label" size="2" weight="bold">
                Password
              </Text>
              <TextField.Root
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleLogin()}
                size="3"
                variant="surface"
              />
            </Flex>
            <Button size="3" loading={loading} onClick={handleLogin}>
              Login
            </Button>
          </Flex>
        </Flex>
      </Card>
    </div>
  );
}
