export default {
  app: 'Totsugeki',
  bracketForm: {
    title: 'Créer une nouvelle bracket',
    nameLabel: 'Nom de la bracket',
    namePlaceholder: 'Weekly #',
  },
  generic: {
    submit: 'Envoyer',
    cancel: 'Annuler',
    about: 'Contact',
    registerLogin: 'Se connecter',
    email: 'Email',
    username: "Nom d'utilisateur",
    password: 'Mot de passe',
    confirmPassword: 'Confirmer le mot de passe',
    register: "S'inscrire",
    profile: 'Profil',
    logout: 'Déconnexion',
    delete: 'Supprimer',
  },
  error: {
    invalidEmail: 'Email invalide',
    required: 'Le champs est requis',
    passwordMissmatch: 'Les mots de passe ne correspondent pas',
    minimum: 'Le champ doit comporter au minimum {min} caractères',
    weakPassword:
      'Le mot de passe fourni est trop faible. Veuillez fournir un mot de passe fort.',
    unknownEmail: 'Email inconnu',
    communication:
      'Une erreur de communication est survenue. Veuillez faire une nouvelle tentative',
    badPassword: 'Le mot de passe ne correspond pas.',
  },
  navbar: {
    myBrackets: 'Mes brackets',
  },
  playerRegistrationForm: {
    title: 'Enregistrer un nouveau joueur',
    newPlayerPlaceholder: 'Nom du joueur',
    minimum: '{min} joueurs minimum',
  },
  registration: {
    bracketNameLabel: 'Bracket',
    startBracket: 'Démarrer',
  },
  playerSeeder: {
    title: 'Seeding',
    hint: 'Déplacer les joueurs pour mettre à jour le seeding',
    empty: "Aucun joueur n'a été enregistré...",
    removeAllPlayers: 'Enlever tous les joueurs',
  },
  bracketView: {
    winnerBracket: 'Winner bracket',
    loserBracket: 'Loser bracket',
    hint: 'Veuillez cliquer un match pour entrer un résultat',
    unsavedWarning:
      'Ce tournoi ne sera pas sauvé. Veuillez vous connectez pour sauver le tournoi.',
    saveBracket: 'Sauvegarder la bracket!',
  },
  loginModal: {
    title: 'Connection',
    text1: "Vous n'avez pas encore de compte ?",
    text2: 'Inscrivez-vous maintenant!',
  },
  signup: {
    register: "S'inscrire",
  },
  user: {
    dashboard: {
      deleteAccount: 'Supprimer le compte',
      deleteMyAccount: 'Supprimer mon compte',
    },
  },
  deleteModal: {
    title: 'Supprimer le compte?',
    confirmWithMail: 'Veuillez tapez {email} pour continuer.',
    matchError: "L'email fourni n'est pas correspond pas à votre email",
  },
  login: 'Vous êtes maintenant connecté',
  logout: 'Déconnexion réussie',
  about: 'Totsugeki est un outil pour créer et manager des brackets.',
}
