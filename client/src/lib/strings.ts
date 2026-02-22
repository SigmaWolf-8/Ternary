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

import { PLATFORM } from "@shared/constants";

export const STRINGS = {
  hero: {
    title: "The World's First",
    titleHighlight: "Ternary Computing",
    titleSuffix: "Platform",
    description: `${PLATFORM.DENSITY_ADVANTAGE}% more information per digit. Femtosecond-precision timing. Post-quantum encryption. A complete Rust kernel with virtual machine, network stack, and binary compatibility layer -- all shipping today.`,
    ctaButton: "Join the Waitlist",
    ctaPending: "Joining...",
    ctaHelper: "No spam. Unsubscribe anytime.",
    viewSource: "View Source",
    liveDemo: "Live Demo",
    whitepaper: "Whitepaper",
    successTitle: "You're on the list!",
    successMessage: "We'll send you SDK access details, documentation links, and priority updates. Check your inbox soon.",
  },
  stats: {
    density: { value: `+${PLATFORM.DENSITY_ADVANTAGE}`, suffix: "%", label: "vs Binary Density" },
    tests: { value: PLATFORM.TESTS_PASSING, label: "Tests Passing" },
    milestones: { value: PLATFORM.MILESTONES, label: "Milestones Complete" },
    opcodes: { value: String(PLATFORM.VM_OPCODES), label: "VM Opcodes" },
  },
  platform: {
    badge: "Complete Platform",
    title: "Everything You Need to Build on Ternary",
    description: `From kernel primitives to application-layer protocols -- a fully integrated ternary computing stack, production-tested with ${PLATFORM.TESTS_PASSING} passing tests.`,
  },
  architecture: {
    badge: "Full-Stack Architecture",
    title: "Built From the Ground Up",
    description: "Five integrated layers spanning hardware abstraction to application services. Every layer is production-tested, binary-compatible, and designed for the post-quantum era.",
  },
  components: {
    badge: "Ship Today",
    title: "Deployable Components",
    description: "Every component is built, tested, and ready for integration. Not a roadmap -- this is what exists right now.",
  },
  performance: {
    badge: "Proven Results",
    title: "Why Ternary Wins",
    description: "Not theoretical advantages -- measured, tested, and verifiable performance improvements you can see in our live demo.",
    demoButton: "Verify It Yourself -- Live Demo",
  },
  codeSnippet: {
    badge: "Try It Now",
    title: "One API Call Away",
    description: `${PLATFORM.API_ENDPOINTS} live endpoints. No SDK required. Start converting ternary operations with a single HTTP request.`,
    exploreButton: `Explore All ${PLATFORM.API_ENDPOINTS} Endpoints`,
  },
  markets: {
    badge: "Market Opportunity",
    title: "Built for Industries That Demand More",
    description: "Targeted deployments with measurable ROI across sectors where efficiency, security, and compliance are non-negotiable.",
  },
  changelog: {
    badge: "Active Development",
    title: "Recent Updates",
    description: "Continuous development on the Ternary kernel and platform.",
    viewAll: "View All Commits",
  },
  developerCta: {
    title: "Request Developer Preview",
    description: "Get early access to the PlenumNET SDK, developer documentation, and direct support from the core team. Be among the first to build applications on ternary infrastructure.",
    ctaButton: "Apply for SDK Access",
    ctaPending: "Submitting...",
    bookDemo: "Book a Demo",
    successTitle: "Application Received!",
    successMessage: "Our team will review your request and reach out within 48 hours with SDK access credentials and onboarding documentation.",
  },
  footer: {
    tagline: "The world's first ternary computing platform. Post-quantum security, 59% density advantage, shipping today.",
    copyright: "All Rights Reserved and Preserved",
    company: "Capomastro Holdings Ltd",
  },
  badges: {
    productionReady: "Production Ready",
    testsPassing: `${PLATFORM.TESTS_PASSING} Tests Passing`,
    postQuantum: "Post-Quantum Secure",
  },
} as const;

export type Strings = typeof STRINGS;
