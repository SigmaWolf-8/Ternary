import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { AlertCircle, ArrowLeft } from "lucide-react";
import { Link } from "wouter";

export default function NotFound() {
  return (
    <div className="flex-1 flex items-center justify-center bg-background">
      <Card className="w-full max-w-md mx-4">
        <CardContent className="pt-6">
          <div className="flex mb-4 gap-2">
            <AlertCircle className="h-8 w-8 text-destructive shrink-0" />
            <h1 className="text-2xl font-bold" data-testid="text-404">404 Page Not Found</h1>
          </div>
          <p className="text-sm text-muted-foreground mb-4">
            The page you're looking for doesn't exist.
          </p>
          <Link href="/">
            <Button variant="outline" size="sm" data-testid="button-go-home">
              <ArrowLeft className="w-3 h-3" />
              Back to File Cabinet
            </Button>
          </Link>
        </CardContent>
      </Card>
    </div>
  );
}
