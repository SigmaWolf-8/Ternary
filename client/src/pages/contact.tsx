/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ArrowLeft, Mail, MapPin, Github, Send } from "lucide-react";
import { motion, useInView } from "framer-motion";
import { useState, useRef } from "react";
import { Link } from "wouter";

function AnimatedSection({ children, delay = 0 }: { children: React.ReactNode; delay?: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay }}
    >
      {children}
    </motion.div>
  );
}

const subjects = [
  { value: "general", label: "General Inquiry" },
  { value: "partnership", label: "Partnership" },
  { value: "sdk-access", label: "SDK Access" },
  { value: "demo-request", label: "Demo Request" },
  { value: "press", label: "Press" },
];

export default function ContactPage() {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [subject, setSubject] = useState("");
  const [message, setMessage] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const selectedSubject = subjects.find((s) => s.value === subject);
    const subjectLine = selectedSubject ? `PlenumNET - ${selectedSubject.label}` : "PlenumNET Inquiry";
    const body = `Name: ${name}\nEmail: ${email}\n\n${message}`;
    const mailtoUrl = `mailto:Rsalvi@Salvigroup.com?subject=${encodeURIComponent(subjectLine)}&body=${encodeURIComponent(body)}`;
    window.location.href = mailtoUrl;
  };

  return (
    <div className="min-h-screen bg-background" data-testid="page-contact">
      <div className="max-w-7xl mx-auto px-5 py-8">
        <div className="mb-8">
          <Button
            variant="ghost"
            size="sm"
            asChild
            data-testid="link-back-home"
            aria-label="Back to home page"
          >
            <Link href="/">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Home
            </Link>
          </Button>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="mb-12"
        >
          <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
            Get in Touch
          </Badge>
          <h1 className="text-4xl md:text-5xl font-bold mb-4" data-testid="text-contact-title">
            Contact PlenumNET
          </h1>
          <p className="text-lg text-muted-foreground max-w-3xl" data-testid="text-contact-subtitle">
            Have questions about ternary computing, partnership opportunities, or SDK access? We'd love to hear from you.
          </p>
        </motion.div>

        <div className="grid lg:grid-cols-3 gap-8">
          <div className="lg:col-span-2">
            <AnimatedSection delay={0.1}>
              <Card className="p-6 md:p-8 border-primary/10" data-testid="section-contact-form">
                <h2 className="text-2xl font-bold mb-6">Send a Message</h2>
                <form onSubmit={handleSubmit} className="space-y-5" data-testid="form-contact">
                  <div className="grid sm:grid-cols-2 gap-4">
                    <div>
                      <label htmlFor="contact-name" className="text-sm font-medium mb-1.5 block">
                        Name
                      </label>
                      <Input
                        id="contact-name"
                        placeholder="Your name"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        required
                        data-testid="input-contact-name"
                      />
                    </div>
                    <div>
                      <label htmlFor="contact-email" className="text-sm font-medium mb-1.5 block">
                        Email
                      </label>
                      <Input
                        id="contact-email"
                        type="email"
                        placeholder="your@email.com"
                        value={email}
                        onChange={(e) => setEmail(e.target.value)}
                        required
                        data-testid="input-contact-email"
                      />
                    </div>
                  </div>

                  <div>
                    <label htmlFor="contact-subject" className="text-sm font-medium mb-1.5 block">
                      Subject
                    </label>
                    <Select value={subject} onValueChange={setSubject}>
                      <SelectTrigger data-testid="select-contact-subject">
                        <SelectValue placeholder="Select a subject" />
                      </SelectTrigger>
                      <SelectContent>
                        {subjects.map((s) => (
                          <SelectItem key={s.value} value={s.value} data-testid={`option-subject-${s.value}`}>
                            {s.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div>
                    <label htmlFor="contact-message" className="text-sm font-medium mb-1.5 block">
                      Message
                    </label>
                    <Textarea
                      id="contact-message"
                      placeholder="Tell us about your project or inquiry..."
                      value={message}
                      onChange={(e) => setMessage(e.target.value)}
                      rows={6}
                      required
                      data-testid="textarea-contact-message"
                    />
                  </div>

                  <Button type="submit" data-testid="button-contact-submit">
                    <Send className="w-4 h-4 mr-2" />
                    Send Message
                  </Button>
                </form>
              </Card>
            </AnimatedSection>
          </div>

          <div className="space-y-6">
            <AnimatedSection delay={0.15}>
              <Card className="p-6 border-primary/10" data-testid="section-contact-info">
                <h3 className="text-lg font-semibold mb-4">Contact Information</h3>
                <div className="space-y-4">
                  <div className="flex items-start gap-3">
                    <Mail className="w-5 h-5 text-primary flex-shrink-0 mt-0.5" />
                    <div>
                      <p className="text-sm font-medium">Email</p>
                      <a
                        href="mailto:Rsalvi@Salvigroup.com"
                        className="text-sm text-primary hover:underline"
                        data-testid="link-contact-email"
                      >
                        Rsalvi@Salvigroup.com
                      </a>
                    </div>
                  </div>
                  <div className="flex items-start gap-3">
                    <MapPin className="w-5 h-5 text-primary flex-shrink-0 mt-0.5" />
                    <div>
                      <p className="text-sm font-medium">Office</p>
                      <p className="text-sm text-muted-foreground" data-testid="text-contact-location">
                        Alberta, Canada
                      </p>
                    </div>
                  </div>
                </div>
              </Card>
            </AnimatedSection>

            <AnimatedSection delay={0.2}>
              <Card className="p-6 border-primary/10" data-testid="section-social-links">
                <h3 className="text-lg font-semibold mb-4">Connect With Us</h3>
                <div className="space-y-3">
                  <Button variant="outline" asChild className="w-full justify-start" data-testid="link-social-github">
                    <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer">
                      <Github className="w-4 h-4 mr-2" />
                      GitHub Repository
                    </a>
                  </Button>
                  <Button variant="outline" asChild className="w-full justify-start" data-testid="link-social-email">
                    <a href="mailto:Rsalvi@Salvigroup.com">
                      <Mail className="w-4 h-4 mr-2" />
                      Rsalvi@Salvigroup.com
                    </a>
                  </Button>
                </div>
              </Card>
            </AnimatedSection>
          </div>
        </div>
      </div>
    </div>
  );
}
