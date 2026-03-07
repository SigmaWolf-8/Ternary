/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

// PlenumNET TDNS Resolver — Content Script
// Detects .plm links and intercepts failed .plm navigations

(function() {
  'use strict';

  // Intercept clicks on .plm links
  document.addEventListener('click', (e) => {
    const link = e.target.closest('a[href]');
    if (!link) return;

    try {
      const url = new URL(link.href, window.location.href);
      if (url.hostname.endsWith('.plm')) {
        e.preventDefault();
        e.stopPropagation();
        const plmName = url.hostname;
        const resolveUrl = browser.runtime.getURL(
          `resolve.html?name=${encodeURIComponent(plmName)}`
        );
        window.location.href = resolveUrl;
      }
    } catch (ex) {
      // Not a valid URL
    }
  }, true);

  // If we're on a DNS error page for a .plm domain, redirect to resolver
  if (document.title.includes('DNS') || document.title.includes('not found') ||
      document.title.includes("can't be reached") || document.title.includes('ERR_NAME')) {
    try {
      const hostname = window.location.hostname;
      if (hostname.endsWith('.plm')) {
        const resolveUrl = browser.runtime.getURL(
          `resolve.html?name=${encodeURIComponent(hostname)}`
        );
        window.location.replace(resolveUrl);
      }
    } catch (ex) {}
  }
})();
