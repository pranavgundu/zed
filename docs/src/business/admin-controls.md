---
title: Admin Controls - Zed Business
description: Configure AI, collaboration, and data sharing settings for your entire Zed Business organization.
---

# Admin Controls

Owners and admins can configure settings that apply to every member of the organization.

These controls cover Zed's server-side features — things that route through Zed's
infrastructure. They don't cover bring-your-own-key (BYOK) configurations or
third-party extensions, since Zed is open source and those work independently of
Zed's servers.

## Accessing Admin Controls

<!-- TODO: document exact location in dashboard before launch -->

Admin controls are available to owners and admins from the organization dashboard at
[dashboard.zed.dev](https://dashboard.zed.dev).

---

## Collaboration

You can disable Zed's real-time collaboration features for the entire organization.
This covers [Channels](../collaboration/channels.md), shared projects, and voice chat —
the features available in the
[Collaboration Panel](../collaboration/overview.md).

When collaboration is disabled, members won't see collaboration features in their Zed
client.

<!-- TODO: confirm exact set of collaboration features covered by this toggle before launch -->

## Hosted AI Models

You can control which of Zed's hosted AI models are available to members. Options are:

- Disable all Zed-hosted models entirely, so members must use their own API keys via
  [Providers](../ai/llm-providers.md) if they want AI features
- Enable or disable access by model provider (Anthropic, OpenAI, Google, etc.)

This control only applies to Zed's hosted model service. Members who bring their own
API keys are not affected.

<!-- TODO: confirm exact model provider controls available at launch -->

## Edit Predictions

You can disable Zed's Edit Predictions for all members of the organization.

## Data Sharing with Zed

You can block members from sharing data with Zed for product improvement purposes.
This covers:

- Edit prediction feedback and training data
- Agent panel interactions shared with Zed

This is enforced at the org level — members can't opt back in individually.

<!-- TODO: confirm exact scope of data sharing controls before launch -->
