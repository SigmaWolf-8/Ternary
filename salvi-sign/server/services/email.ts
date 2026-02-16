import { Resend } from "resend";

const resend = new Resend(process.env.Resend_API_KEY);

const FROM_ADDRESS = "SalviSign <onboarding@resend.dev>";

interface SigningEmailParams {
  recipientId: string;
  recipientEmail: string;
  recipientName: string;
  envelopeId: string;
  envelopeName: string;
  senderName: string;
  role: string;
}

export async function sendSigningEmail(params: SigningEmailParams) {
  const { recipientId, recipientEmail, recipientName, envelopeId, envelopeName, senderName, role } = params;

  const host = process.env.REPLIT_DEV_DOMAIN
    ? `https://${process.env.REPLIT_DEV_DOMAIN}`
    : process.env.REPLIT_DOMAINS
      ? `https://${process.env.REPLIT_DOMAINS.split(",")[0]}`
      : "http://localhost:5000";

  const signUrl = `${host}/sign/${envelopeId}/${recipientId}`;

  const actionText = role === "signer" ? "sign" : role === "witness" ? "witness" : "view";

  const html = `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
</head>
<body style="margin:0;padding:0;background-color:#0a0a0a;font-family:'Inter',Arial,sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#0a0a0a;padding:40px 20px;">
    <tr>
      <td align="center">
        <table width="560" cellpadding="0" cellspacing="0" style="background-color:#141414;border:1px solid #2a2a2a;border-radius:8px;">
          <tr>
            <td style="padding:32px 40px 24px;border-bottom:1px solid #2a2a2a;">
              <span style="font-size:22px;font-weight:700;color:#d4a843;letter-spacing:1px;">SALVISIGN</span>
            </td>
          </tr>
          <tr>
            <td style="padding:32px 40px;">
              <p style="margin:0 0 16px;font-size:15px;color:#e0e0e0;line-height:1.6;">
                Hello ${recipientName},
              </p>
              <p style="margin:0 0 24px;font-size:15px;color:#b0b0b0;line-height:1.6;">
                <strong style="color:#e0e0e0;">${senderName}</strong> has requested you to ${actionText} the document
                <strong style="color:#e0e0e0;">&ldquo;${envelopeName}&rdquo;</strong>.
              </p>
              <table cellpadding="0" cellspacing="0" style="margin:0 0 28px;">
                <tr>
                  <td style="background-color:#d4a843;border-radius:6px;padding:12px 32px;">
                    <a href="${signUrl}" style="color:#0a0a0a;font-size:14px;font-weight:600;text-decoration:none;letter-spacing:0.5px;text-transform:uppercase;">
                      ${role === "viewer" ? "View Document" : "Review & Sign"}
                    </a>
                  </td>
                </tr>
              </table>
              <p style="margin:0 0 8px;font-size:12px;color:#777;">
                Or copy this link into your browser:
              </p>
              <p style="margin:0 0 24px;font-size:12px;color:#d4a843;word-break:break-all;">
                ${signUrl}
              </p>
            </td>
          </tr>
          <tr>
            <td style="padding:20px 40px;border-top:1px solid #2a2a2a;">
              <p style="margin:0;font-size:11px;color:#555;line-height:1.5;">
                This email was sent by SalviSign on behalf of ${senderName}.
                If you did not expect this document, you can safely ignore this email.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>`.trim();

  try {
    const result = await resend.emails.send({
      from: FROM_ADDRESS,
      to: recipientEmail.toLowerCase(),
      subject: `${senderName} requests your signature on "${envelopeName}"`,
      html,
    });

    if (result.error) {
      console.error(`[Email] Failed to send to ${recipientEmail}:`, result.error.message);
      return { success: false, error: result.error.message };
    }

    console.log(`[Email] Sent to ${recipientEmail} for envelope ${envelopeId}, id=${result.data?.id}`);
    return { success: true, data: result.data };
  } catch (error: any) {
    console.error(`[Email] Failed to send to ${recipientEmail}:`, error.message);
    return { success: false, error: error.message };
  }
}

export async function sendEnvelopeEmails(
  envelopeId: string,
  envelopeName: string,
  senderName: string,
  recipients: Array<{ id: string; email: string; name: string; role: string }>
) {
  const results = await Promise.allSettled(
    recipients.map((r) =>
      sendSigningEmail({
        recipientId: r.id,
        recipientEmail: r.email,
        recipientName: r.name,
        envelopeId,
        envelopeName,
        senderName,
        role: r.role,
      })
    )
  );

  const sent = results.filter((r) => r.status === "fulfilled" && (r.value as any).success).length;
  const failed = results.length - sent;
  console.log(`[Email] Envelope ${envelopeId}: ${sent} sent, ${failed} failed`);
  return { sent, failed, total: results.length };
}
