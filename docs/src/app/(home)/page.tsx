  import Link from "next/link";

    export default function HomePage() {
      return (
        <main className="flex min-h-screen flex-col items-center justify-center bg-fd-background text-fd-foreground">
          {/* Hero */}
          <section className="mx-auto flex max-w-4xl flex-col items-center gap-6 px-4 py-24 text-center">
            <span className="rounded-full border border-fd-border bg-fd-card px-4 py-1.5 text-sm font-medium text-fd-muted-foreground">
              Open Source Release Automation
            </span>

            <h1 className="text-4xl font-extrabold tracking-tight sm:text-5xl md:text-6xl">
              Ship to <span className="text-fd-primary">every platform</span> with
              one command
            </h1>

            <p className="max-w-2xl text-lg text-fd-muted-foreground sm:text-xl">
              system-releaser detects your language, builds optimized binaries,
              generates package&nbsp;manager manifests, and publishes GitHub
              releases&nbsp;&mdash; automatically.
            </p>

            <div className="flex flex-wrap justify-center gap-4">
              <Link
                href="/docs"
                className="rounded-lg bg-fd-primary px-6 py-3 font-semibold text-fd-primary-foreground transition hover:opacity-90"
              >
                Get Started
              </Link>
              <Link
                href="/docs/cli/release"
                className="rounded-lg border border-fd-border bg-fd-card px-6 py-3 font-semibold text-fd-foreground transition hover:bg-fd-accent"
              >
                CLI Reference
              </Link>
            </div>
          </section>

          {/* Feature cards */}
          <section className="mx-auto grid max-w-5xl grid-cols-1 gap-6 px-4 pb-24 sm:grid-cols-2 lg:grid-cols-3">
            {[
              {
                title: "23 Languages",
                desc: "Rust, Go, Python, Node.js, C/C++, Java, Kotlin, Swift, Zig, and more — detected and built automatically.",
              },
              {
                title: "14 Package Managers",
                desc: "Homebrew, APT, Pacman, Nix, WinGet, Chocolatey, Scoop, and more — manifests generated for you.",
              },
              {
                title: "Secure by Default",
                desc: "No secrets in CLI args. Atomic file operations. Path-traversal protection. CWE-hardened throughout.",
              },
              {
                title: "Cross-Platform Builds",
                desc: "Build for Linux, macOS, and Windows from your CI. ARM64 and x86_64 supported out of the box.",
              },
              {
                title: "One Config File",
                desc: "A single releaser.yaml drives your entire release pipeline — builds, packaging, and publishing.",
              },
              {
                title: "CI/CD Ready",
                desc: "Generate GitHub Actions workflows with `releaser init-ci`. Ship on every push to main.",
              },
            ].map((f) => (
              <div
                key={f.title}
                className="rounded-xl border border-fd-border bg-fd-card p-6"
              >
                <h3 className="mb-2 text-lg font-bold">{f.title}</h3>
                <p className="text-sm text-fd-muted-foreground">{f.desc}</p>
              </div>
            ))}
          </section>
        </main>
      );
    }