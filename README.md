# `@malolebrin/cv-normalizer`

Module natif (Rust + NAPI-RS) pour **normaliser et compresser des CV côté Node.js**.

- **Images (`image/png`, `image/jpeg`, `image/jpg`)** → conversion en **PDF 1 page** : décodage, downscale basique, recompression JPEG, puis encapsulation dans un PDF minimal.  
- **PDF (`application/pdf`, `application/x-pdf`)** → **validation du header `%PDF-`** puis retour des octets inchangés (hook prêt pour un futur pipeline d’optimisation PDF).  
- **Autres mime types** → pour l’instant, **pass-through** (octets inchangés).

Le module est pensé pour être appelé depuis un backend Node/Strapi au moment de la réception des CV.

---

## Installation

Une fois publié sur npm :

```bash
pnpm add @malolebrin/cv-normalizer
```

En local dans ce repo :

```bash
pnpm install
pnpm build
```

Le build génère le binaire natif `cv-normalizer.*.node` et le fichier de binding `index.js`.

---

## API Node / TypeScript

Signature générée (`index.d.ts`) :

```ts
export declare function normalizeCvToPdf(
  bytes: Uint8Array,
  mime: string,
): number[]
```

Usage typique côté Node :

```ts
import { normalizeCvToPdf } from '@malolebrin/cv-normalizer'

// buffer: Buffer ou Uint8Array contenant le CV
// mime: string ('image/png', 'image/jpeg', 'application/pdf', etc.)
const out = normalizeCvToPdf(buffer, mime)

// Le binding renvoie un Array<number>, on le remet en Buffer pour Node
const pdfBuffer = Buffer.from(out)
```

### Comportement détaillé

- **Images (`image/png`, `image/jpeg`, `image/jpg`, `image/pjpeg`)**  
  - Décodage via la crate Rust `image`.  
  - Downscale si nécessaire pour que le plus grand côté ≤ 2000 px.  
  - Ré-encodage en JPEG (qualité ≈ 80).  
  - Génération d’un **PDF 1 page** qui dessine l’image sur toute la page.

- **PDF (`application/pdf`, `application/x-pdf`)**  
  - Vérifie que les octets commencent par `"%PDF-"`.  
  - Si ce n’est pas le cas → lève une erreur NAPI (`code: InvalidArg`).  
  - Sinon → retourne les octets **tels quels** (future extension : recompression/optimisation PDF).

- **Autres mime types**  
  - Les octets sont simplement renvoyés inchangés.

---

## Script de démo en ligne de commande

Pour tester la normalisation sur de **vrais fichiers** (images ou PDF), un script simple est fourni :

```bash
pnpm build
pnpm demo /chemin/vers/mon_cv.png
pnpm demo /chemin/vers/mon_cv.pdf
```

Ce script (`scripts/normalize-file.cjs`) :

- détecte ou utilise le `mimeType` passé en argument,  
- appelle `normalizeCvToPdf`,  
- écrit un fichier de sortie `mon_cv.normalized.pdf` à côté du fichier d’entrée,  
- affiche la taille avant/après et le pourcentage de gain (ou de croissance) de taille.

Usage détaillé :

```bash
node scripts/normalize-file.cjs <inputPath> [mimeType] [outputPath]
```

Exemples :

```bash
# Image PNG → PDF
node scripts/normalize-file.cjs ./fixtures/cv.png

# Image JPEG avec mime explicite
node scripts/normalize-file.cjs ./fixtures/cv.jpg image/jpeg

# PDF existant (validation + pass-through)
node scripts/normalize-file.cjs ./fixtures/cv.pdf
```

---

## Développement

### Prérequis

- **Rust** récent (édition 2021).  
- **Node.js** ≥ 18 (la CI tourne aujourd’hui sur Node 20/22/24).  
- **pnpm** (gestionnaire de paquets utilisé dans ce repo).

### Commandes principales

```bash
# Installer les dépendances JS
pnpm install

# Build du module natif (toutes cibles activées dans package.json)
pnpm build

# Lancer les tests (AVA)
pnpm test

# Lint JS/TS
pnpm lint

# Formatage Rust / JS / TOML
pnpm format
```

Les tests actuels vérifient notamment :

- le comportement **no-op** sur un petit PDF valide,  
- la remontée d’erreur NAPI (`InvalidArg`) sur une image PNG volontairement invalide (couverture du chemin d’erreur image).

---

## Intégration typique (Strapi / backend Node)

Dans un backend Node/Strapi, le pattern recommandé est :

```ts
import { normalizeCvToPdf } from '@malolebrin/cv-normalizer'

async function normalizeIncomingCv(file: { buffer: Buffer; mime: string }) {
  const out = normalizeCvToPdf(file.buffer, file.mime)
  const pdfBuffer = Buffer.from(out)

  return {
    ...file,
    buffer: pdfBuffer,
    size: pdfBuffer.length,
    mime: 'application/pdf',
  }
}
```

Ce helper peut être appelé dans un **controller** ou un **lifecycle** Strapi juste avant la persistance du CV pour que tous les CV stockés soient déjà **en PDF normalisé**.

---

## CI & Release

- **CI GitHub Actions**  
  - Build, lint et tests sont exécutés sur une matrice Node.js / OS (Linux, macOS, Windows) en utilisant **pnpm** pour les dépendances JS et `cargo` pour la partie Rust.  
  - Les artefacts `.node` / `.wasm` sont prébuildés pour plusieurs plateformes via `@napi-rs/cli`.

- **Publication npm**  
  - La publication est gérée via la CI à partir des tags git (`npm version` + `git push`).  
  - Assure-toi d’avoir configuré `NPM_TOKEN` dans les secrets GitHub du repo.  
  - Ne pas lancer `npm publish` manuellement : la pipeline GitHub Actions se charge de publier les packages précompilés.

