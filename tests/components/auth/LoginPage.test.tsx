import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { LoginPage } from "@/components/auth/LoginPage";

const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
  },
}));

const tMock = vi.fn((key: string) => key);
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: tMock }),
}));

const postMock = vi.fn();
const setAuthTokenMock = vi.fn();
const generateWebAdminTokenMock = vi.fn();

vi.mock("@tauri-icons/icon.png", () => ({
  default: "/mocked-icon.png",
}));

vi.mock("@/lib/api/web-client", () => ({
  post: (...args: unknown[]) => postMock(...args),
  setAuthToken: (...args: unknown[]) => setAuthTokenMock(...args),
}));

vi.mock("@/lib/api", () => ({
  authApi: {
    generateWebAdminToken: () => generateWebAdminTokenMock(),
  },
}));

const renderLoginPage = (props: { onLogin?: () => void } = {}) => {
  return render(<LoginPage onLogin={props.onLogin ?? vi.fn()} />);
};

describe("LoginPage Component", () => {
  beforeEach(() => {
    toastSuccessMock.mockReset();
    toastErrorMock.mockReset();
    postMock.mockReset();
    setAuthTokenMock.mockReset();
    generateWebAdminTokenMock.mockReset();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("renders the login form with input and buttons", () => {
    renderLoginPage();

    expect(screen.getByLabelText("login.token")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "login.revealToken" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "login.submit" }),
    ).toBeInTheDocument();
  });

  it("shows error toast when submitting empty token", async () => {
    renderLoginPage();

    fireEvent.click(screen.getByRole("button", { name: "login.submit" }));

    await waitFor(() => {
      expect(toastErrorMock).toHaveBeenCalledWith("login.tokenRequired");
    });
  });

  it("reveals token and fills input on success", async () => {
    generateWebAdminTokenMock.mockResolvedValue("revealed-secret-token");

    renderLoginPage();

    fireEvent.click(screen.getByRole("button", { name: "login.revealToken" }));

    await waitFor(() => {
      expect(generateWebAdminTokenMock).toHaveBeenCalledTimes(1);
      expect(toastSuccessMock).toHaveBeenCalledWith("login.tokenRevealed");
    });

    const input = screen.getByPlaceholderText(
      "login.tokenPlaceholder",
    ) as HTMLInputElement;
    expect(input.value).toBe("revealed-secret-token");
  });

  it("shows error toast when reveal token fails", async () => {
    generateWebAdminTokenMock.mockRejectedValue(
      new Error("Server unreachable"),
    );

    renderLoginPage();

    fireEvent.click(screen.getByRole("button", { name: "login.revealToken" }));

    await waitFor(() => {
      expect(toastErrorMock).toHaveBeenCalledWith("login.tokenRevealFailed");
    });
  });

  it("submits valid token, calls setAuthToken and onLogin", async () => {
    postMock.mockResolvedValue({ valid: true });
    const onLogin = vi.fn();

    renderLoginPage({ onLogin });

    const input = screen.getByPlaceholderText("login.tokenPlaceholder");
    fireEvent.change(input, { target: { value: "valid-token" } });

    fireEvent.click(screen.getByRole("button", { name: "login.submit" }));

    await waitFor(() => {
      expect(postMock).toHaveBeenCalledWith("/auth/verify", {
        token: "valid-token",
      });
      expect(setAuthTokenMock).toHaveBeenCalledWith("valid-token");
      expect(toastSuccessMock).toHaveBeenCalledWith("login.success");
      expect(onLogin).toHaveBeenCalledTimes(1);
    });
  });

  it("shows error toast when token is invalid (API returns valid: false)", async () => {
    postMock.mockResolvedValue({ valid: false });

    renderLoginPage();

    const input = screen.getByPlaceholderText("login.tokenPlaceholder");
    fireEvent.change(input, { target: { value: "invalid-token" } });

    fireEvent.click(screen.getByRole("button", { name: "login.submit" }));

    await waitFor(() => {
      expect(postMock).toHaveBeenCalledWith("/auth/verify", {
        token: "invalid-token",
      });
      expect(toastErrorMock).toHaveBeenCalledWith("login.error");
    });
    expect(setAuthTokenMock).not.toHaveBeenCalled();
  });

  it("shows error toast on network error during submit", async () => {
    postMock.mockRejectedValue(new Error("Network error"));

    renderLoginPage();

    const input = screen.getByPlaceholderText("login.tokenPlaceholder");
    fireEvent.change(input, { target: { value: "some-token" } });

    fireEvent.click(screen.getByRole("button", { name: "login.submit" }));

    await waitFor(() => {
      expect(toastErrorMock).toHaveBeenCalledWith("login.error");
    });
  });

  it("renders the desktop logo image", () => {
    renderLoginPage();
    const logo = screen.getByRole("img", { name: "login.logoAlt" });
    expect(logo).toBeInTheDocument();
    expect(logo.getAttribute("src")).toContain("icon");
  });

  it("disables buttons during reveal and submit", async () => {
    let resolveReveal: (value: string) => void;
    const revealPromise = new Promise<string>(
      (resolve) => (resolveReveal = resolve),
    );
    generateWebAdminTokenMock.mockReturnValue(revealPromise);

    let resolvePost: (value: { valid: boolean }) => void;
    const postPromise = new Promise<{ valid: boolean }>(
      (resolve) => (resolvePost = resolve),
    );
    postMock.mockReturnValue(postPromise);

    renderLoginPage();

    const revealButton = screen.getByRole("button", { name: "login.revealToken" });
    const submitButton = screen.getByRole("button", { name: "login.submit" });
    const input = screen.getByPlaceholderText(
      "login.tokenPlaceholder",
    ) as HTMLInputElement;

    // During reveal: input and reveal button disabled, submit stays enabled
    fireEvent.click(revealButton);

    await waitFor(() => {
      expect(input).toBeDisabled();
      expect(revealButton).toBeDisabled();
    });
    // Submit button is NOT disabled during reveal
    expect(submitButton).not.toBeDisabled();

    resolveReveal!("token");
    await waitFor(() => expect(input).not.toBeDisabled());

    // During submit: all disabled
    fireEvent.change(input, { target: { value: "my-token" } });
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(input).toBeDisabled();
      expect(revealButton).toBeDisabled();
      expect(submitButton).toBeDisabled();
    });

    resolvePost!({ valid: true });
    await waitFor(() => expect(input).not.toBeDisabled());
  });
});
