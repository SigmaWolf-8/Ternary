import { Link } from "wouter";
import { Box, Github, Mail } from "lucide-react";

const footerLinks = {
  Platform: [
    { label: "Ternary Kernel", href: "https://github.com/SigmaWolf-8/Ternary" },
    { label: "PlenumDB Demo", href: "/ternarydb" },
    { label: "Salvi API", href: "/api-demo" },
    { label: "Kong Gateway", href: "/kong-konnect" },
  ],
  Developers: [
    { label: "Whitepaper", href: "/whitepaper" },
    { label: "API Demo", href: "/api-demo" },
    { label: "Documentation", href: "/docs" },
    { label: "GitHub", href: "https://github.com/SigmaWolf-8/Ternary" },
  ],
  Company: [
    { label: "About", href: "/about" },
    { label: "Contact", href: "/contact" },
    { label: "CNSA 2.0 Compliance", href: "/compliance" },
  ],
  Legal: [
    { label: "Privacy", href: "/privacy" },
    { label: "Terms", href: "/terms" },
    { label: "Security", href: "/security" },
    { label: "Acceptable Use", href: "/aup" },
  ],
};

export function MarketingFooter() {
  return (
    <footer className="bg-background border-t border-primary/10 py-16" data-testid="footer">
      <div className="max-w-7xl mx-auto px-5">
        <div className="grid grid-cols-2 md:grid-cols-5 gap-8 mb-12">
          <div className="col-span-2 md:col-span-1">
            <Link
              href="/"
              className="flex items-center gap-2 text-primary font-bold text-xl mb-4"
              data-testid="link-footer-logo"
            >
              <Box className="w-6 h-6" />
              <span>PlenumNET</span>
            </Link>
            <p className="text-sm text-muted-foreground mb-4">
              The world's first ternary computing platform. Post-quantum
              security, 59% density advantage, shipping today.
            </p>
            <div className="flex gap-3">
              <a
                href="https://github.com/SigmaWolf-8/Ternary"
                target="_blank"
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-github"
                aria-label="GitHub repository"
              >
                <Github className="w-5 h-5" />
              </a>
              <a
                href="mailto:Rsalvi@Salvigroup.com"
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-email"
                aria-label="Send email"
              >
                <Mail className="w-5 h-5" />
              </a>
            </div>
          </div>

          {Object.entries(footerLinks).map(([category, links]) => (
            <div key={category}>
              <h4 className="font-semibold mb-4 text-foreground">{category}</h4>
              <ul className="space-y-2">
                {links.map((link) => (
                  <li key={link.label}>
                    {link.href.startsWith("/") && !link.href.startsWith("/#") ? (
                      <Link
                        href={link.href}
                        className="text-sm text-muted-foreground hover:text-primary transition-colors"
                      >
                        {link.label}
                      </Link>
                    ) : (
                      <a
                        href={link.href}
                        className="text-sm text-muted-foreground hover:text-primary transition-colors"
                        target={link.href.startsWith("http") ? "_blank" : undefined}
                        rel={link.href.startsWith("http") ? "noopener noreferrer" : undefined}
                      >
                        {link.label}
                      </a>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="pt-8 border-t border-primary/10 text-center text-sm text-muted-foreground">
          <p>All Rights Reserved and Preserved | &copy; Capomastro Holdings Ltd 2026</p>
        </div>
      </div>
    </footer>
  );
}
