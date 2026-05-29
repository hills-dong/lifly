import { useLogin } from "@refinedev/core";
import { useState } from "react";

export function LoginPage() {
  const { mutate: login, isPending } = useLogin();
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    login(
      { username, password },
      {
        onSuccess: (data) => {
          if (!(data as any)?.success) setError("用户名或密码错误");
        },
        onError: () => setError("用户名或密码错误"),
      },
    );
  };

  return (
    <div
      style={{
        minHeight: "100vh",
        display: "grid",
        placeItems: "center",
        background: "#f3f4f6",
        fontFamily: "system-ui",
      }}
    >
      <form
        onSubmit={submit}
        style={{
          background: "#fff",
          padding: 32,
          borderRadius: 8,
          boxShadow: "0 1px 3px rgba(0,0,0,.1)",
          width: 320,
        }}
      >
        <h1 style={{ fontSize: 20, marginTop: 0 }}>Lifly 运维后台</h1>
        <label style={{ display: "block", fontWeight: 600, marginBottom: 4 }}>用户名</label>
        <input
          name="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          style={{ width: "100%", padding: "8px", marginBottom: 12, boxSizing: "border-box" }}
        />
        <label style={{ display: "block", fontWeight: 600, marginBottom: 4 }}>密码</label>
        <input
          name="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          style={{ width: "100%", padding: "8px", marginBottom: 12, boxSizing: "border-box" }}
        />
        {error && (
          <div data-testid="login-error" style={{ color: "#dc2626", marginBottom: 12 }}>
            {error}
          </div>
        )}
        <button
          type="submit"
          disabled={isPending}
          style={{
            width: "100%",
            padding: "10px",
            background: "#2563eb",
            color: "#fff",
            border: "none",
            borderRadius: 4,
            cursor: "pointer",
          }}
        >
          {isPending ? "登录中…" : "登录"}
        </button>
      </form>
    </div>
  );
}
