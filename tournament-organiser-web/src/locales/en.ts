export default {
  app: 'Totsugeki',
  bracketForm: {
    title: 'Create new bracket',
    nameLabel: 'Bracket name',
    namePlaceholder: 'Weekly #',
  },
  generic: {
    submit: 'Submit',
    cancel: 'Cancel',
    registerLogin: 'Register / Login',
    about: 'About',
    email: 'Email',
    username: 'Username',
    password: 'Password',
    confirmPassword: 'Confirm password',
    register: 'Register',
    profile: 'Profile',
    logout: 'Logout',
    delete: 'Delete',
  },
  error: {
    invalidEmail: 'This field must be a valid email',
    required: 'This field is required',
    passwordMissmatch: 'Passwords must match',
    minimum: 'At least {min} characters are required',
    weakPassword:
      'Provided password is too weak. Please provide a stronger password',
    unknownEmail: 'Unknown email',
    communication: 'An internal error happened. Please try again.',
    badPassword: 'Wrong password was provided',
  },
  navbar: {
    myBrackets: 'My brackets',
  },
  playerRegistrationForm: {
    title: 'Register players',
    newPlayerPlaceholder: 'Player name',
    minimum: '{min} players minimum',
  },
  registration: {
    bracketNameLabel: 'Bracket',
    startBracket: 'Start bracket',
  },
  playerSeeder: {
    title: 'Seeding',
    hint: 'Drag and drop players to update the seeding',
    empty: 'No players registered...',
    removeAllPlayers: 'Remove all players',
  },
  bracketView: {
    winnerBracket: 'Winner bracket',
    loserBracket: 'Loser bracket',
    hint: 'Click on matches to enter results',
    unsavedWarning: 'This bracket is currently unsaved. Please log in to save.',
    saveBracket: 'Save bracket!',
  },
  loginModal: {
    title: 'Login',
    text1: "Don't have an account yet?",
    text2: 'Register now!',
  },
  user: {
    dashboard: {
      deleteAccount: 'Delete account',
      deleteMyAccount: 'Delete my account',
    },
  },
  deleteModal: {
    title: 'Delete account?',
    confirmWithMail: 'Please type {email} to continue.',
    matchError: 'Email must match your email',
  },
  login: 'Successful login',
  logout: 'Successful logout',
  about: 'Totsugeki is a tool for creating and managing brackets.',
}
