import { motion } from "framer-motion";
import { ArrowLeft } from "lucide-react";
import { Link } from "wouter";
import { Button } from "@/components/ui/button";
import InstallExtensionCard from "@/components/InstallExtensionCard";

export default function InstallExtensionPage() {
  return (
    <div className="min-h-screen bg-background" data-testid="page-install-extension">
      <div className="max-w-7xl mx-auto px-5 py-16 md:py-24">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
        >
          <Link href="/">
            <Button variant="ghost" size="sm" className="mb-8" data-testid="button-back-home">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back
            </Button>
          </Link>

          <h1 className="text-3xl md:text-4xl font-bold mb-3" data-testid="text-page-title">
            Install TDNS Browser Extension
          </h1>
          <p className="text-muted-foreground text-lg max-w-2xl mb-10">
            One command installs the .plm resolver in every browser on your machine.
            Auto-detects Chrome, Edge, Brave, Firefox, Opera, and Vivaldi.
          </p>

          <InstallExtensionCard />
        </motion.div>
      </div>
    </div>
  );
}
