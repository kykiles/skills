package cmdargs

// Profile — общий набор аргументов для запуска утилиты.
type Profile struct {
	Base []string
}

// NewProfile создаёт профиль с базовыми флагами и флагами пользователя.
// Пустые флаги пользователя пропускаются.
func NewProfile(userFlags ...string) Profile {
	base := make([]string, 0, 16)
	base = append(base, "--quiet", "--color=never")
	base = append(base, userFlags...)
	return Profile{Base: WithoutEmpty(base)}
}

// Command возвращает аргументы конкретной команды.
func (p Profile) Command(extra ...string) []string {
	return WithoutEmpty(Build(p.Base, extra...))
}
